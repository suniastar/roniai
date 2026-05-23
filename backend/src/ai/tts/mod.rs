pub mod sample;

use crate::ai::tts::sample::Sample;
use crate::args::Args;
use anyhow::{Context, Result};
use hf_hub::api::sync::ApiBuilder;
use qwen3_tts::{AudioBuffer, Language, Qwen3TTS, SynthesisOptions, auto_device};
use std::time::Instant;
use tracing::debug;

const REPO_LOW: &str = "Qwen/Qwen3-TTS-12Hz-0.6B-Base";
const REPO_HIGH: &str = "Qwen/Qwen3-TTS-12Hz-1.7B-Base";
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
    pub fn new(args: &Args) -> Result<Self> {
        let repo = match args.tts_low_quality() {
            true => REPO_LOW.to_string(),
            false => REPO_HIGH.to_string(),
        };
        let api_repo = ApiBuilder::from_env()
            .with_progress(true)
            .build()?
            .model(repo);
        api_repo.get(CONFIG.into())?;
        api_repo.get(VOCAB.into())?;
        api_repo.get(MERGES.into())?;
        let model_path = api_repo.get(FILE.into())?;
        api_repo.get(TOKEN_FILE.into())?;
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
        let device = auto_device()?;
        let model = Qwen3TTS::from_pretrained_with_tokenizer(&self.model_directory, None, device)?;

        let start_req = Instant::now();
        let audio_req = sythesize_voice(&model, clone, req, Language::German)?;
        debug!(
            "synthesize question voice clone in {}ms",
            start_req.elapsed().as_millis()
        );

        let start_res = Instant::now();
        let audio_res = sythesize_voice(&model, Sample::Mrsroni, res, Language::German)?;
        debug!(
            "synthesize response voice clone in {}ms",
            start_res.elapsed().as_millis()
        );

        Ok((audio_req, audio_res))
    }
}

fn sythesize_voice(
    model: &Qwen3TTS,
    sample: Sample,
    text: &str,
    language: Language,
) -> Result<AudioBuffer> {
    let options = SynthesisOptions {
        seed: Some(42),
        ..SynthesisOptions::default()
    };
    let (ref_text_req, ref_audio_req) = sample.ref_audio_ref_text()?;
    let prompt_req = model.create_voice_clone_prompt(&ref_audio_req, Some(ref_text_req))?;
    model.synthesize_voice_clone(text, &prompt_req, language, Some(options.clone()))
}

#[cfg(test)]
const SAMPLE_TEXT_DE: &str = "Dieser Satz soll testen, ob die Technik funktioniert.";
#[cfg(test)]
const SAMPLE_TEXT_EN: &str = "This sentence should prove the software's functionality.";
#[cfg(test)]
const SAMPLE_TEXT_JP: &str = "この文は、そのソフトウェアの機能を証明するはずです。";
#[cfg(test)]
#[test]
fn test_low_synthesis() {
    let dir = std::env::current_dir()
        .expect("no current dir")
        .join("target");
    let tts = TTS::new(&Args::sample_args_with_low_tts()).expect("failed to init tts");
    let device = auto_device().expect("failed to init auto device");
    let model = Qwen3TTS::from_pretrained_with_tokenizer(&tts.model_directory, None, device)
        .expect("failed to model");

    for sample in Sample::all() {
        println!("Synthesize sample {}", sample);
        let de = sythesize_voice(&model, *sample, SAMPLE_TEXT_DE, Language::German)
            .expect("failed to synthesize german");
        de.save(dir.join(format!("{}_de_low.wav", sample)))
            .expect("failed to save german");
        println!("german done");
        let en = sythesize_voice(&model, *sample, SAMPLE_TEXT_EN, Language::English)
            .expect("failed to synthesize english");
        en.save(dir.join(format!("{}_en_low.wav", sample)))
            .expect("failed to save english");
        println!("english done");
        let jp = sythesize_voice(&model, *sample, SAMPLE_TEXT_JP, Language::Japanese)
            .expect("failed to synthesize japanese");
        jp.save(dir.join(format!("{}_jp_low.wav", sample)))
            .expect("failed to save japanese");
        println!("japanese done");
    }
}

#[cfg(test)]
#[test]
fn test_high_synthesis() {
    let dir = std::env::current_dir()
        .expect("no current dir")
        .join("target");
    let tts = TTS::new(&Args::sample_args_with_high_tts()).expect("failed to init tts");
    let device = auto_device().expect("failed to init auto device");
    let model = Qwen3TTS::from_pretrained_with_tokenizer(&tts.model_directory, None, device)
        .expect("failed to model");

    for sample in Sample::all() {
        println!("Synthesize sample {}", sample);
        let de = sythesize_voice(&model, *sample, SAMPLE_TEXT_DE, Language::German)
            .expect("failed to synthesize german");
        de.save(dir.join(format!("{}_de_high.wav", sample)))
            .expect("failed to save german");
        println!("german done");
        let en = sythesize_voice(&model, *sample, SAMPLE_TEXT_EN, Language::English)
            .expect("failed to synthesize english");
        en.save(dir.join(format!("{}_en_high.wav", sample)))
            .expect("failed to save english");
        println!("english done");
        let jp = sythesize_voice(&model, *sample, SAMPLE_TEXT_JP, Language::Japanese)
            .expect("failed to synthesize japanese");
        jp.save(dir.join(format!("{}_jp_high.wav", sample)))
            .expect("failed to save japanese");
        println!("japanese done");
    }
}
