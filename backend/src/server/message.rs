use anyhow::{Error, Result};
use axum::extract::ws::Message as WsMessage;
use rmp_serde::to_vec_named as rmp_serialize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "eval")]
    EvaluatingPrompt { id: u64, prompt: String },
    #[serde(rename = "say")]
    Say {
        id: u64,
        text: String,
        // TODO add audio
    },
}

impl TryFrom<&Message> for WsMessage {
    type Error = Error;

    fn try_from(value: &Message) -> Result<WsMessage> {
        let binary = rmp_serialize(value)?;
        Ok(WsMessage::Binary(binary.into()))
    }
}
