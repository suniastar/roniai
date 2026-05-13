mod embedded;

use crate::embedded::AudioSample;
use anyhow::{Result, ensure};
use qwen3_tts::{AudioBuffer, Qwen3TTS, SynthesisOptions, auto_device, Language};

fn main() -> Result<()> {
    let device = auto_device()?;
    let model = Qwen3TTS::from_pretrained(
        "/hdd/home/frederik/ComfyUI/models/qwen-tts/Qwen3-TTS-12Hz-1.7B-Base",
        device,
    )?;

    // text
    let text = "Junge! Immer diese scheiß Frage. Ich weiß, dass ich die schlechteste Pilotin auf Narco City bin, ja? Lasst mich doch endlich mal damit in Ruhe.";
    let options = SynthesisOptions {
        seed: Some(42),
        ..SynthesisOptions::default()
    };

    // clone
    let ref_audio = AudioBuffer::load("src/embedded/roni_sample.wav")?;
    let ref_text = AudioSample::Roni.txt();
    ensure!(model.has_speech_encoder(), "speech encoder is required");

    let prompt = model.create_voice_clone_prompt(&ref_audio, Some(ref_text))?;
    let (audio, codes) = model.synthesize_voice_clone_debug(text, &prompt, Language::German, Some(options))?;
    audio.save("output.wav")?;

    // Print semantic tokens to check for EOS (2150)
    let semantic_tokens: Vec<u32> = codes.iter().map(|f| f[0]).collect();
    eprintln!(
        "Semantic tokens ({} frames): {:?}",
        codes.len(),
        &semantic_tokens
    );
    let has_eos = semantic_tokens.contains(&qwen3_tts::CODEC_EOS_TOKEN_ID);
    eprintln!(
        "Contains EOS ({}): {has_eos}",
        qwen3_tts::CODEC_EOS_TOKEN_ID
    );
    eprintln!(
        "ICL clone: {:.2}s, {} samples → output_clone_icl.wav",
        audio.duration(),
        audio.len()
    );

    Ok(())
}
