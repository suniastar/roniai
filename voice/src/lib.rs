use candle_core::safetensors::{load_buffer, save};
use candle_core::{Device, Error as CandleError, Tensor};
use qwen3_tts::VoiceClonePrompt;
use rmp_serde::decode::Error as DeserializerError;
use rmp_serde::encode::Error as SerializerError;
use rmp_serde::{from_read as rmp_deserialize, to_vec as rmp_serialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::temp_dir;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs::{read, remove_file, write};
use std::io::{Error as IoError, Read};
use std::path::PathBuf;

const KEY_SPEAKER: &str = "s";
const KEY_CODES: &str = "c";

pub fn save_cloned_voice(prompt: VoiceClonePrompt, file: &PathBuf) -> Result<(), VoiceError> {
    let temp_dir = temp_dir();
    let temp_file = temp_dir.join("cloned_voice.tensors");
    if temp_file.exists() {
        return Err(VoiceError::TempFileExists);
    }
    let tensors = HashMap::<&'static str, Tensor>::from_iter(
        [
            (KEY_SPEAKER, Some(prompt.speaker_embedding)),
            (KEY_CODES, prompt.ref_codes),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|t| (k, t))),
    );
    save(&tensors, &temp_file)?;
    let tensor_data = read(&temp_file)?;
    remove_file(temp_file)?;
    let cv = ClonedVoice {
        tensor_data,
        ref_text_ids: prompt.ref_text_ids,
    };
    let bytes = rmp_serialize(&cv)?;
    write(file, &bytes)?;
    Ok(())
}

pub fn load_cloned_voice<R>(read: R, device: &Device) -> Result<VoiceClonePrompt, VoiceError>
where
    R: Read,
{
    let cv: ClonedVoice = rmp_deserialize(read)?;
    let mut tensors = load_buffer(&cv.tensor_data, device)?;
    let speaker_embedding = tensors
        .remove(KEY_SPEAKER)
        .ok_or(VoiceError::MissingSpeakerEmbedding)?;
    let ref_codes = tensors.remove(KEY_CODES);
    let ref_text_ids = cv.ref_text_ids;
    Ok(VoiceClonePrompt {
        speaker_embedding,
        ref_codes,
        ref_text_ids,
    })
}

#[derive(Debug)]
#[repr(u8)]
pub enum VoiceError {
    TempFileExists,
    MissingSpeakerEmbedding,
    IO(IoError),
    Serialize(SerializerError),
    Deserialize(DeserializerError),
    Candle(CandleError),
}

impl Display for VoiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::TempFileExists => write!(f, "temp file exists"),
            Self::MissingSpeakerEmbedding => write!(f, "missing speaker embedding"),
            Self::IO(e) => write!(f, "I/O error: {e}"),
            Self::Serialize(e) => write!(f, "Serialization error: {e}"),
            Self::Deserialize(e) => write!(f, "Deserialize error: {e}"),
            Self::Candle(e) => write!(f, "Candle error: {e}"),
        }
    }
}

impl Error for VoiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IO(e) => Some(e),
            Self::Serialize(e) => Some(e),
            Self::Deserialize(e) => Some(e),
            Self::Candle(e) => Some(e),
            _ => None,
        }
    }
}

impl From<IoError> for VoiceError {
    fn from(value: IoError) -> Self {
        Self::IO(value)
    }
}

impl From<SerializerError> for VoiceError {
    fn from(value: SerializerError) -> Self {
        Self::Serialize(value)
    }
}

impl From<DeserializerError> for VoiceError {
    fn from(value: DeserializerError) -> Self {
        Self::Deserialize(value)
    }
}

impl From<CandleError> for VoiceError {
    fn from(value: CandleError) -> Self {
        Self::Candle(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ClonedVoice {
    tensor_data: Vec<u8>,
    ref_text_ids: Option<Vec<u32>>,
}
