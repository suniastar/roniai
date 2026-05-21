use anyhow::{Error, Result, bail};
use hound::{SampleFormat, WavReader};
use qwen3_tts::AudioBuffer;
use rand::Rng;
use serde::{Deserialize, Serialize};

const MASK: u32 = u8::MAX as u32;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename = "snake_case")]
#[repr(u8)]
pub enum Sample {
    Roni = 0,
    Json,
}

impl TryFrom<u8> for Sample {
    type Error = Error;

    fn try_from(value: u8) -> Result<Sample> {
        const COUNT: u8 = 1;
        match value % COUNT {
            0 => Ok(Sample::Json),
            _ => bail!("Unknown sample {}", value),
        }
    }
}

impl Sample {
    pub fn random<R>(rng: &mut R) -> Result<Self>
    where
        R: Rng,
    {
        let s: u8 = (rng.next_u32() & MASK) as u8;
        Ok(s.try_into()?)
    }

    pub fn ref_audio_ref_text(&self) -> Result<(&'static str, AudioBuffer)> {
        let text = self.txt();
        let wav = self.wav();
        let audio = load_wav(wav)?;
        Ok((text, audio))
    }

    fn txt(&self) -> &'static str {
        match self {
            Self::Roni => include_str!("samples/roni.txt"),
            Self::Json => include_str!("samples/json.txt"),
        }
    }

    fn wav(&self) -> &'static [u8] {
        match self {
            Self::Roni => include_bytes!("samples/roni.wav"),
            Self::Json => include_bytes!("samples/json.wav"),
        }
    }
}

fn load_wav(bytes: &[u8]) -> Result<AudioBuffer> {
    let reader = WavReader::new(bytes)?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    let channels = spec.channels as usize;

    let samples: Vec<f32> = match spec.sample_format {
        SampleFormat::Float => reader
            .into_samples::<f32>()
            .collect::<Result<Vec<_>, _>>()?,
        SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            let max_val = (1 << (bits - 1)) as f32;
            reader
                .into_samples::<i32>()
                .map(|s| s.map(|v| v as f32 / max_val))
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    // Convert to mono by averaging channels
    let mono_samples = if channels > 1 {
        samples
            .chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples
    };

    Ok(AudioBuffer::new(mono_samples, sample_rate))
}
