use crate::ai::AIResponse;
use anyhow::{Error, Result};
use axum::extract::ws::Message as WsMessage;
use qwen3_tts::AudioBuffer;
use rmp_serde::to_vec_named as rmp_serialize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "eval")]
    EvaluatingPrompt { id: u64, req_txt: String },
    #[serde(rename = "say")]
    Say {
        id: u64,
        req_wav: Audio,
        res_txt: String,
        res_wav: Audio,
    },
}

impl Message {
    pub fn eval(id: u64, req_txt: String) -> Self {
        Message::EvaluatingPrompt { id, req_txt }
    }
    pub fn say(id: u64, res: AIResponse) -> Self {
        Message::Say {
            id,
            req_wav: res.request_wav.into(),
            res_txt: res.response_text,
            res_wav: res.response_wav.into(),
        }
    }
}

impl TryFrom<&Message> for WsMessage {
    type Error = Error;

    fn try_from(value: &Message) -> Result<WsMessage> {
        let binary = rmp_serialize(value)?;
        Ok(WsMessage::Binary(binary.into()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audio {
    samples: Vec<f32>,
    sample_rate: u32,
}

impl From<AudioBuffer> for Audio {
    fn from(values: AudioBuffer) -> Self {
        Self {
            samples: values.samples,
            sample_rate: values.sample_rate,
        }
    }
}
