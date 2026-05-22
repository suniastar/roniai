pub mod sample;

use crate::ai::tts::sample::Sample;
use anyhow::{Context, Result};
use hf_hub::api::sync::ApiBuilder;
use qwen3_tts::{AudioBuffer, Language, Qwen3TTS, SynthesisOptions, auto_device};
use std::time::Instant;
use tracing::debug;

const REPO: &str = "Qwen/Qwen3-TTS-12Hz-1.7B-Base";
const CONFIG: &str = "config.json";
const VOCAB: &str = "vocab.json";
const MERGES: &str = "merges.txt";
const FILE: &str = "model.safetensors";
const TOKEN_FILE: &str = "speech_tokenizer/model.safetensors";

#[derive(Debug)]
pub struct TTS {
    model_directory: String,
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
        let model = Qwen3TTS::from_pretrained_with_tokenizer(&model_directory, None, device)?;
        drop(model);

        Ok(Self { model_directory })
    }

    pub fn prompt(
        &mut self,
        clone: Sample,
        req: &str,
        res: &str,
    ) -> Result<(AudioBuffer, AudioBuffer)> {
        let options = SynthesisOptions {
            seed: Some(42),
            ..SynthesisOptions::default()
        };
        let device = auto_device()?;
        let model = Qwen3TTS::from_pretrained_with_tokenizer(&self.model_directory, None, device)?;
        let start = Instant::now();

        // req
        let (ref_text_req, ref_audio_req) = clone.ref_audio_ref_text()?;
        let prompt_req = model.create_voice_clone_prompt(&ref_audio_req, Some(ref_text_req))?;
        let audio_req = model.synthesize_voice_clone(
            req,
            &prompt_req,
            Language::German,
            Some(options.clone()),
        )?;
        debug!(
            "synthesize question voice clone in {}ms",
            start.elapsed().as_millis()
        );

        // resp
        let (ref_text_res, ref_audio_res) = Sample::Roni.ref_audio_ref_text()?;
        let prompt_res = model.create_voice_clone_prompt(&ref_audio_res, Some(ref_text_res))?;
        let audio_res =
            model.synthesize_voice_clone(res, &prompt_res, Language::German, Some(options))?;
        debug!(
            "synthesize response voice clone in {}ms",
            start.elapsed().as_millis()
        );

        Ok((audio_req, audio_res))
    }
}
