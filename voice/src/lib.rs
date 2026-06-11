use candle_core::safetensors::{load_buffer, save};
use candle_core::{Device, Error as CandleError, Tensor};
use qwen3_tts::VoiceClonePrompt;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

const KEY_SPEAKER: &str = "s";
const KEY_CODES: &str = "c";

pub fn save_cloned_voice(prompt: VoiceClonePrompt, file: &PathBuf) -> Result<(), VoiceError> {
    let tensors = HashMap::<&'static str, Tensor>::from_iter(
        [
            (KEY_SPEAKER, Some(prompt.speaker_embedding)),
            (KEY_CODES, prompt.ref_codes),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|t| (k, t))),
    );
    save(&tensors, file)?;
    Ok(())
}

pub fn load_cloned_voice(bytes: &[u8], device: &Device) -> Result<VoiceClonePrompt, VoiceError> {
    let mut tensors = load_buffer(bytes, device)?;
    let speaker_embedding = tensors
        .remove(KEY_SPEAKER)
        .ok_or(VoiceError::MissingSpeakerEmbedding)?;
    let ref_codes = tensors.remove(KEY_CODES);
    Ok(VoiceClonePrompt {
        speaker_embedding,
        ref_codes,
        ref_text_ids: None,
    })
}

#[derive(Debug)]
#[repr(u8)]
pub enum VoiceError {
    TempFileExists,
    MissingSpeakerEmbedding,
    Candle(CandleError),
}

impl Display for VoiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::TempFileExists => write!(f, "temp file exists"),
            Self::MissingSpeakerEmbedding => write!(f, "missing speaker embedding"),
            Self::Candle(e) => write!(f, "Candle error: {e}"),
        }
    }
}

impl Error for VoiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Candle(e) => Some(e),
            _ => None,
        }
    }
}

impl From<CandleError> for VoiceError {
    fn from(value: CandleError) -> Self {
        Self::Candle(value)
    }
}
