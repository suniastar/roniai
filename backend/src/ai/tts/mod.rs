pub mod sample;

use crate::ai::tts::sample::Sample;
use crate::args::Args;
use anyhow::{Context, Error, Result};
use hf_hub::api::sync::ApiBuilder;
use qwen3_tts::{AudioBuffer, Language, Qwen3TTS, SynthesisOptions, auto_device};
use std::fs::read_to_string;
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
const CONFIG: &str = "config.json";
const TOKEN_CONFIG: &str = "tokenizer_config.json";
const VOCAB: &str = "vocab.json";
const MERGES: &str = "merges.txt";
const FILE: &str = "model.safetensors";
const TOKEN_FILE: &str = "speech_tokenizer/model.safetensors";

/// Pre-tokenizer regex matching Python's `Qwen2Converter` (from `convert_slow_tokenizer.py`).
const PRETOKENIZE_REGEX: &str = r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+(?!\S)|\s+";

#[derive(Debug)]
pub struct TTS {
    model_directory: String,
    tokenizer_path: String,
    low: bool,
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
        api_repo.get(CONFIG)?;
        let token_config_path = api_repo
            .get(TOKEN_CONFIG)
            .map(|f| Some(f))
            .unwrap_or_default();
        let vocab_path = api_repo.get(VOCAB)?;
        let merges_path = api_repo.get(MERGES)?;
        let model_path = api_repo.get(FILE)?;
        api_repo.get(TOKEN_FILE)?;
        let model_directory: String = model_path
            .parent()
            .context("no parent dir")?
            .canonicalize()?
            .to_str()
            .context("invalid path")?
            .into();

        // pre-build the tokenizer
        let bpe = BPE::from_file(
            vocab_path.to_str().context("invalid vocab path")?,
            merges_path.to_str().context("invalid merges path")?,
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
        if let Some(config_path) = token_config_path {
            let content = read_to_string(config_path)?;
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
        let tokenizer_json = model_path
            .parent()
            .context("no parent dir")?
            .join("tokenizer_gen.json");
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
        let audio_req = sythesize_voice(&model, self.low, clone, req, Language::German)?;
        debug!(
            "synthesize question voice clone in {}ms",
            start_req.elapsed().as_millis()
        );

        let start_res = Instant::now();
        let audio_res = sythesize_voice(&model, self.low, Sample::Mrsroni, res, Language::German)?;
        debug!(
            "synthesize response voice clone in {}ms",
            start_res.elapsed().as_millis()
        );

        Ok((audio_req, audio_res))
    }
}

fn sythesize_voice(
    model: &Qwen3TTS,
    low: bool,
    sample: Sample,
    text: &str,
    language: Language,
) -> Result<AudioBuffer> {
    let options = SynthesisOptions {
        seed: Some(42),
        ..SynthesisOptions::default()
    };
    let prompt = sample.voice_clone_prompt(model.device(), low)?;
    model.synthesize_voice_clone(text, &prompt, language, Some(options.clone()))
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
        let de = sythesize_voice(&model, true, *sample, SAMPLE_TEXT_DE, Language::German)
            .expect("failed to synthesize german");
        de.save(dir.join(format!("{}_de_low.wav", sample)))
            .expect("failed to save german");
        println!("german done");
        let en = sythesize_voice(&model, true, *sample, SAMPLE_TEXT_EN, Language::English)
            .expect("failed to synthesize english");
        en.save(dir.join(format!("{}_en_low.wav", sample)))
            .expect("failed to save english");
        println!("english done");
        let jp = sythesize_voice(&model, true, *sample, SAMPLE_TEXT_JP, Language::Japanese)
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
        let de = sythesize_voice(&model, false, *sample, SAMPLE_TEXT_DE, Language::German)
            .expect("failed to synthesize german");
        de.save(dir.join(format!("{}_de_high.wav", sample)))
            .expect("failed to save german");
        println!("german done");
        let en = sythesize_voice(&model, false, *sample, SAMPLE_TEXT_EN, Language::English)
            .expect("failed to synthesize english");
        en.save(dir.join(format!("{}_en_high.wav", sample)))
            .expect("failed to save english");
        println!("english done");
        let jp = sythesize_voice(&model, false, *sample, SAMPLE_TEXT_JP, Language::Japanese)
            .expect("failed to synthesize japanese");
        jp.save(dir.join(format!("{}_jp_high.wav", sample)))
            .expect("failed to save japanese");
        println!("japanese done");
    }
}
