use anyhow::{Error, Result, bail};
use hound::{SampleFormat, WavReader};
use qwen3_tts::AudioBuffer;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use twitch_api::types::UserIdRef;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename = "snake_case")]
#[repr(u8)]
pub enum Sample {
    Mrsroni = 0,
    AylinCel,
    Jerzy,
    Onlyjson,
    RubySpell,
    Whitecharline,
}

impl Display for Sample {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Mrsroni => write!(f, "mrsroni"),
            Self::AylinCel => write!(f, "aylin_cel"),
            Self::Jerzy => write!(f, "jerzy"),
            Self::Onlyjson => write!(f, "onlyjson"),
            Self::RubySpell => write!(f, "ruby_spell"),
            Self::Whitecharline => write!(f, "whitecharline"),
        }
    }
}

impl Sample {
    pub fn all() -> &'static [Sample] {
        &[
            Self::Mrsroni,
            Self::AylinCel,
            Self::Jerzy,
            Self::Onlyjson,
            Self::RubySpell,
            Self::Whitecharline,
        ]
    }

    pub fn from_user_id_or_random<R>(user_id: &UserIdRef, rng: &mut R) -> Self
    where
        R: Rng,
    {
        Self::try_from(user_id).unwrap_or_else(|_| Self::random(rng))
    }

    pub fn random<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        let r = rng.next_u32() as usize;
        let l = Self::all().len() - 1;
        let i = (r % l) + 1;
        let s = Self::all()[i];
        s
    }

    pub fn ref_audio_ref_text(&self) -> Result<(&'static str, AudioBuffer)> {
        let text = self.txt();
        let wav = self.wav();
        let audio = load_wav(wav)?;
        Ok((text, audio))
    }

    fn txt(&self) -> &'static str {
        match self {
            Self::Mrsroni => include_str!("samples/mrsroni.txt"),
            Self::AylinCel => include_str!("samples/aylin_cel.txt"),
            Self::Jerzy => include_str!("samples/jerzy.txt"),
            Self::Onlyjson => include_str!("samples/onlyjson.txt"),
            Self::RubySpell => include_str!("samples/ruby_spell.txt"),
            Self::Whitecharline => include_str!("samples/whitecharline.txt"),
        }
    }

    fn wav(&self) -> &'static [u8] {
        match self {
            Self::Mrsroni => include_bytes!("samples/mrsroni.wav"),
            Self::AylinCel => include_bytes!("samples/aylin_cel.wav"),
            Self::Jerzy => include_bytes!("samples/jerzy.wav"),
            Self::Onlyjson => include_bytes!("samples/onlyjson.wav"),
            Self::RubySpell => include_bytes!("samples/ruby_spell.wav"),
            Self::Whitecharline => include_bytes!("samples/whitecharline.wav"),
        }
    }
}

impl TryFrom<&UserIdRef> for Sample {
    type Error = Error;

    fn try_from(value: &UserIdRef) -> Result<Sample> {
        match value.as_str() {
            "501133499" => Ok(Self::Mrsroni),
            "1252866592" => Ok(Self::AylinCel),
            "765006197" => Ok(Self::Jerzy),
            "198939622" => Ok(Self::Onlyjson),
            "773366406" => Ok(Self::RubySpell),
            "157225932" => Ok(Self::Whitecharline),
            _ => bail!("Unknown user id reference {}", value),
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
