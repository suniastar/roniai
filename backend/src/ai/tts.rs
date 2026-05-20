use anyhow::{Context, Result};
use candle_core::Device;
use hf_hub::api::sync::ApiBuilder;
use qwen3_tts::{AudioBuffer, Language, Qwen3TTS, SynthesisOptions, VoiceClonePrompt, auto_device};
use std::time::Instant;
use tracing::debug;
use voice::load_cloned_voice;

const REPO: &str = "Qwen/Qwen3-TTS-12Hz-1.7B-Base";
const CONFIG: &str = "config.json";
const VOCAB: &str = "vocab.json";
const MERGES: &str = "merges.txt";
const FILE: &str = "model.safetensors";
const TOKEN_FILE: &str = "speech_tokenizer/model.safetensors";

pub struct TTS {
    model_directory: String,
    device: Device,
}

impl TTS {
    pub fn new() -> Result<Self> {
        let api = ApiBuilder::from_env().with_progress(true).build()?;
        api.model(REPO.into()).get(CONFIG.into())?;
        api.model(REPO.into()).get(VOCAB.into())?;
        api.model(REPO.into()).get(MERGES.into())?;
        let model_path = api.model(REPO.into()).get(FILE.into())?;
        api.model(REPO.into()).get(TOKEN_FILE.into())?;
        let model_directory: String = model_path
            .parent()
            .context("no parent dir")?
            .canonicalize()?
            .to_str()
            .context("invalid path")?
            .into();

        // load model and drop to cache it in ram
        let device = auto_device()?;
        let model =
            Qwen3TTS::from_pretrained_with_tokenizer(&model_directory, None, device.clone())?;
        drop(model);

        Ok(Self {
            model_directory,
            device,
        })
    }

    pub fn prompt(&mut self, text: &str) -> Result<AudioBuffer> {
        let options = SynthesisOptions {
            seed: Some(42),
            ..SynthesisOptions::default()
        };
        let model = Qwen3TTS::from_pretrained_with_tokenizer(
            &self.model_directory,
            None,
            self.device.clone(),
        )?;
        let prompt = Voice::Roni.prompt(&self.device)?;
        let start = Instant::now();
        let audio = model.synthesize_voice_clone(text, &prompt, Language::German, Some(options))?;
        debug!(
            "synthesize voice clone in {}ms",
            start.elapsed().as_millis()
        );

        Ok(audio)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum Voice {
    Roni,
    Json,
    Freddy,
}

impl Voice {
    fn bytes(&self) -> &'static [u8] {
        match self {
            Self::Roni => include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../target/voices/roni.voice"
            )),
            Self::Json => include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../target/voices/json.voice"
            )),
            Self::Freddy => include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../target/voices/freddy.voice"
            )),
        }
    }

    fn prompt(&self, device: &Device) -> Result<VoiceClonePrompt> {
        let bytes = self.bytes();
        let prompt = load_cloned_voice(bytes, device)?;
        Ok(prompt)
    }
}
