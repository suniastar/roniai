pub mod sample;

use crate::ai::tts::sample::Sample;
use crate::args::Args;
use anyhow::{Context, Error, Result};
use hf_hub::api::sync::ApiBuilder;
use qwen3_tts::{AudioBuffer, Language, Qwen3TTS, SynthesisOptions, auto_device};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::Instant;
use tokenizers::models::bpe::BPE;
use tokenizers::normalizers::NFC;
use tokenizers::pre_tokenizers::byte_level::ByteLevel;
use tokenizers::pre_tokenizers::sequence::Sequence;
use tokenizers::pre_tokenizers::split::Split;
use tokenizers::{AddedToken, SplitDelimiterBehavior, Tokenizer};
use tracing::debug;

const REPO_LOW: &str = "Qwen/Qwen3-TTS-12Hz-0.6B-Base";
const REPO_HIGH: &str = "Qwen/Qwen3-TTS-12Hz-1.7B-Base";

/// Pre-tokenizer regex matching Python's `Qwen2Converter` (from `convert_slow_tokenizer.py`).
const PRETOKENIZE_REGEX: &str = r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+(?!\S)|\s+";

#[derive(Debug)]
pub struct TTS {
    model_directory: String,
    tokenizer_path: String,
    seed: u64,
    low: bool,
}

impl TTS {
    pub fn new(args: &Args) -> Result<Self> {
        let repo = match args.tts_low_quality() {
            true => REPO_LOW.to_string(),
            false => REPO_HIGH.to_string(),
        };
        let model_dir = download_repo(repo)?;
        let model_directory = model_dir.to_str().context("invalid model dir")?.to_owned();

        // pre-build the tokenizer
        let bpe = BPE::from_file(
            model_dir
                .join("vocab.json")
                .to_str()
                .context("invalid vocab path")?,
            model_dir
                .join("merges.txt")
                .to_str()
                .context("invalid merges path")?,
        )
        .unk_token("<|endoftext|>".to_string())
        .byte_fallback(false)
        .build()
        .map_err(Error::msg)?;
        let mut tokenizer = Tokenizer::new(bpe);
        tokenizer.with_normalizer(Some(NFC));
        let split = Split::new(PRETOKENIZE_REGEX, SplitDelimiterBehavior::Isolated, false)
            .map_err(Error::msg)?;
        let byte_level = ByteLevel::new(false, false, false);
        tokenizer.with_pre_tokenizer(Some(Sequence::new(vec![split.into(), byte_level.into()])));
        tokenizer.with_post_processor(Some(ByteLevel::new(false, false, false)));
        tokenizer.with_decoder(Some(ByteLevel::new(false, false, false)));
        let token_config_path = model_dir.join("tokenizer_config.json");
        if token_config_path.try_exists()? {
            let content = read_to_string(token_config_path)?;
            let config: serde_json::Value = serde_json::from_str(&content)?;
            let added_tokens_decoder = config
                .get("added_tokens_decoder")
                .and_then(|v| v.as_object());
            if let Some(added_tokens) = added_tokens_decoder {
                let mut special_tokens = Vec::new();
                for (_id_str, token_info) in added_tokens {
                    let content = match token_info.get("content").and_then(|v| v.as_str()) {
                        Some(c) => c,
                        None => continue,
                    };
                    let is_special = token_info
                        .get("special")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if is_special {
                        let mut token = AddedToken::from(content, true);
                        if let Some(lstrip) = token_info.get("lstrip").and_then(|v| v.as_bool()) {
                            token = token.lstrip(lstrip);
                        }
                        if let Some(rstrip) = token_info.get("rstrip").and_then(|v| v.as_bool()) {
                            token = token.rstrip(rstrip);
                        }
                        if let Some(normalized) =
                            token_info.get("normalized").and_then(|v| v.as_bool())
                        {
                            token = token.normalized(normalized);
                        }
                        if let Some(single_word) =
                            token_info.get("single_word").and_then(|v| v.as_bool())
                        {
                            token = token.single_word(single_word);
                        }
                        special_tokens.push(token);
                    }
                }
                if !special_tokens.is_empty() {
                    debug!(
                        "Adding {} special tokens from tokenizer_config.json",
                        special_tokens.len()
                    );
                    tokenizer.add_special_tokens(&special_tokens);
                }
            }
        }
        let tokenizer_json = model_dir.join("tokenizer_gen.json");
        tokenizer.save(&tokenizer_json, false).map_err(Error::msg)?;
        let tokenizer_path = tokenizer_json
            .to_str()
            .context("invalid tokenizer path")?
            .to_owned();

        // load model and drop to cache it in ram
        let device = auto_device()?;
        let model = Qwen3TTS::from_pretrained_with_tokenizer(
            &model_directory,
            Some(&tokenizer_path),
            device,
        )?;
        drop(model);

        Ok(Self {
            model_directory,
            tokenizer_path,
            seed: args.seed(),
            low: args.tts_low_quality(),
        })
    }

    pub fn prompt(
        &mut self,
        clone: Sample,
        req: &str,
        res: &str,
    ) -> Result<(AudioBuffer, AudioBuffer)> {
        let device = auto_device()?;
        let model = Qwen3TTS::from_pretrained_with_tokenizer(
            &self.model_directory,
            Some(&self.tokenizer_path),
            device,
        )?;

        let start_req = Instant::now();
        let audio_req = self.sythesize_voice(&model, clone, req, Language::German)?;
        debug!(
            "synthesize question voice clone in {}ms",
            start_req.elapsed().as_millis()
        );

        let start_res = Instant::now();
        let audio_res = self.sythesize_voice(&model, Sample::Mrsroni, res, Language::German)?;
        debug!(
            "synthesize response voice clone in {}ms",
            start_res.elapsed().as_millis()
        );

        Ok((audio_req, audio_res))
    }

    fn sythesize_voice(
        &self,
        model: &Qwen3TTS,
        sample: Sample,
        text: &str,
        language: Language,
    ) -> Result<AudioBuffer> {
        let options = SynthesisOptions {
            seed: Some(self.seed),
            ..SynthesisOptions::default()
        };
        let prompt = sample.voice_clone_prompt(model.device(), self.low)?;
        model.synthesize_voice_clone(text, &prompt, language, Some(options.clone()))
    }
}

fn download_repo(id: impl Into<String>) -> Result<PathBuf> {
    let repo = ApiBuilder::from_env()
        .with_progress(true)
        .build()?
        .model(id.into());
    repo.get("speech_tokenizer/config.json")?;
    repo.get("speech_tokenizer/configuration.json")?;
    repo.get("speech_tokenizer/model.safetensors")?;
    repo.get("speech_tokenizer/preprocessor_config.json")?;
    repo.get("config.json")?;
    repo.get("generation_config.json")?;
    repo.get("merges.txt")?;
    repo.get("model.safetensors")?;
    repo.get("preprocessor_config.json")?;
    repo.get("tokenizer_config.json")?;
    let vocab = repo.get("vocab.json")?;
    Ok(vocab.parent().context("missing parent")?.to_owned())
}

#[cfg(test)]
const SAMPLE_TEXT: &str = "Nur weil ich nach über 1000 Stunden auf Narco immer noch keinen Heli fliegen kann, müsst ihr mich nicht zwingend Tonne nennen.";

#[cfg(test)]
fn set_up(args: Args) -> (TTS, Qwen3TTS) {
    let tts = TTS::new(&args).expect("failed to init tts");
    let device = auto_device().expect("failed to init device");
    let model = Qwen3TTS::from_pretrained_with_tokenizer(
        &tts.model_directory,
        Some(&tts.tokenizer_path),
        device,
    )
    .expect("failed to init model");
    (tts, model)
}

#[cfg(test)]
fn tear_down(audio: AudioBuffer, name: &str) {
    let path = std::env::current_dir()
        .expect("no current dir")
        .join("target");
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("failed to create target dir");
    }
    audio.save(path.join(name)).expect("failed to save audio");
}

#[cfg(test)]
#[test]
fn test_mrsroni_high() {
    let (tts, model) = set_up(Args::sample_args_with_high_tts());

    let sample = Sample::Mrsroni;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "mrsroni_high.wav");
}

#[cfg(test)]
#[test]
fn test_mrsroni_low() {
    let (tts, model) = set_up(Args::sample_args_with_low_tts());

    let sample = Sample::Mrsroni;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "mrsroni_low.wav");
}

#[cfg(test)]
#[test]
fn test_aylin_cel_high() {
    let (tts, model) = set_up(Args::sample_args_with_high_tts());

    let sample = Sample::AylinCel;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "aylin_cel_high.wav");
}

#[cfg(test)]
#[test]
fn test_aylin_cel_low() {
    let (tts, model) = set_up(Args::sample_args_with_low_tts());

    let sample = Sample::AylinCel;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "aylin_cel_low.wav");
}

#[cfg(test)]
#[test]
fn test_jerzy_high() {
    let (tts, model) = set_up(Args::sample_args_with_high_tts());

    let sample = Sample::Jerzy;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "jerzy_high.wav");
}

#[cfg(test)]
#[test]
fn test_jerzy_low() {
    let (tts, model) = set_up(Args::sample_args_with_low_tts());

    let sample = Sample::Jerzy;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "jerzy_low.wav");
}

#[cfg(test)]
#[test]
fn test_onlyjson_high() {
    let (tts, model) = set_up(Args::sample_args_with_high_tts());

    let sample = Sample::Onlyjson;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "onlyjson_high.wav");
}

#[cfg(test)]
#[test]
fn test_onlyjson_low() {
    let (tts, model) = set_up(Args::sample_args_with_low_tts());

    let sample = Sample::Onlyjson;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "onlyjson_low.wav");
}

#[cfg(test)]
#[test]
fn test_ruby_spell_high() {
    let (tts, model) = set_up(Args::sample_args_with_high_tts());

    let sample = Sample::RubySpell;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "ruby_spell_high.wav");
}

#[cfg(test)]
#[test]
fn test_ruby_spell_low() {
    let (tts, model) = set_up(Args::sample_args_with_low_tts());

    let sample = Sample::RubySpell;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "ruby_spell_low.wav");
}

#[cfg(test)]
#[test]
fn test_whitecharline_high() {
    let (tts, model) = set_up(Args::sample_args_with_high_tts());

    let sample = Sample::Whitecharline;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "whitecharline_high.wav");
}

#[cfg(test)]
#[test]
fn test_whitecharline_low() {
    let (tts, model) = set_up(Args::sample_args_with_low_tts());

    let sample = Sample::Whitecharline;
    let wave = tts.sythesize_voice(&model, sample, SAMPLE_TEXT, Language::German);
    assert!(wave.is_ok(), "voice clone failed {}", wave.unwrap_err());

    tear_down(wave.unwrap(), "whitecharline_low.wav");
}
