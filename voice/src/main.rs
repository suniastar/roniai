use anyhow::{Context, Result, bail};
use hf_hub::api::sync::ApiBuilder;
use qwen3_tts::{AudioBuffer, Qwen3TTS, auto_device};
use std::ffi::OsStr;
use std::fs::{create_dir_all, read_dir, read_to_string};
use std::path::PathBuf;
use std::time::Instant;
use voice::save_cloned_voice;

const IN_DIR: &str = "voice/samples";
const OUT_DIR: &str = "target/voices";
const REPO: &str = "Qwen/Qwen3-TTS-12Hz-1.7B-Base";
const CONFIG: &str = "config.json";
const VOCAB: &str = "vocab.json";
const MERGES: &str = "merges.txt";
const FILE: &str = "model.safetensors";
const TOKEN_FILE: &str = "speech_tokenizer/model.safetensors";

fn main() -> Result<()> {
    let root = std::env::current_dir()?;
    let in_dir = root.join(IN_DIR);
    let out_dir = root.join(OUT_DIR);

    if !in_dir.try_exists()? {
        bail!("input directory {} does not exist", in_dir.display());
    }
    let samples = list_samples(&in_dir)?;
    if !out_dir.try_exists()? {
        create_dir_all(&out_dir)?;
    }

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

    let device = auto_device()?;
    let model = Qwen3TTS::from_pretrained_with_tokenizer(&model_directory, None, device)?;

    for (path_wav, path_txt) in samples.iter() {
        let start = Instant::now();
        let name = path_wav
            .file_name()
            .context("missing filename")?
            .to_str()
            .context("invalid filename")?
            .split(".")
            .next()
            .context("no extension")?;
        println!("Cloning \"{name}\" ...");
        let out_file = out_dir.join(name).with_extension("voice");

        let audio = AudioBuffer::load(path_wav)?;
        let text = read_to_string(path_txt)?;
        let prompt = model.create_voice_clone_prompt(&audio, Some(&text))?;
        save_cloned_voice(prompt, &out_file)?;

        println!(
            "Cloned to \"{}\" complete. Took {}ms",
            out_file.display(),
            start.elapsed().as_millis()
        );
    }

    Ok(())
}

fn list_samples(dir: &PathBuf) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut samples = Vec::<(PathBuf, PathBuf)>::with_capacity(32);
    for entry in read_dir(dir)? {
        let path = entry?.path();
        if let Some(ext) = path.extension()
            && ext != OsStr::new("wav")
        {
            continue;
        }
        let txt = path.with_extension("txt");
        if !txt.exists() {
            continue;
        }
        samples.push((path, txt));
    }
    Ok(samples)
}
