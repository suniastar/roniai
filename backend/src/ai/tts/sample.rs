use anyhow::{Error, Result, bail};
use candle_core::Device;
use qwen3_tts::VoiceClonePrompt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use twitch_api::types::UserIdRef;
use voice::load_cloned_voice;

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
        Self::all()[i]
    }

    pub fn voice_clone_prompt(&self, device: &Device, low: bool) -> Result<VoiceClonePrompt> {
        let bytes = self.vox(low);
        let prompt = load_cloned_voice(bytes, device)?;
        Ok(prompt)
    }

    fn vox(&self, low: bool) -> &'static [u8] {
        match (self, low) {
            (Self::Mrsroni, false) => include_bytes!("sample/mrsroni_high.vox"),
            (Self::Mrsroni, true) => include_bytes!("sample/mrsroni_low.vox"),
            (Self::AylinCel, false) => include_bytes!("sample/aylin_cel_high.vox"),
            (Self::AylinCel, true) => include_bytes!("sample/aylin_cel_low.vox"),
            (Self::Jerzy, false) => include_bytes!("sample/jerzy_high.vox"),
            (Self::Jerzy, true) => include_bytes!("sample/jerzy_low.vox"),
            (Self::Onlyjson, false) => include_bytes!("sample/onlyjson_high.vox"),
            (Self::Onlyjson, true) => include_bytes!("sample/onlyjson_low.vox"),
            (Self::RubySpell, false) => include_bytes!("sample/ruby_spell_high.vox"),
            (Self::RubySpell, true) => include_bytes!("sample/ruby_spell_low.vox"),
            (Self::Whitecharline, false) => include_bytes!("sample/whitecharline_high.vox"),
            (Self::Whitecharline, true) => include_bytes!("sample/whitecharline_low.vox"),
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
