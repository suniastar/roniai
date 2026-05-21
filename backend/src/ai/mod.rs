use crate::ai::llm::LLM;
use crate::ai::tts::TTS;
use crate::ai::tts::sample::Sample;
use crate::state::AppState;
use anyhow::Result;
use qwen3_tts::AudioBuffer;
use rand::Rng;
use std::time::Instant;
use tracing::debug;
use twitch_api::types::UserIdRef;

mod llm;
pub mod tts;

#[derive(Debug)]
pub struct AI {
    state: AppState,
    llm: LLM,
    tts: TTS,
}

impl AI {
    pub fn new(state: AppState) -> Result<Self> {
        let llm = LLM::new()?;
        let tts = TTS::new()?;
        Ok(Self { state, llm, tts })
    }

    pub async fn eval<R>(
        &mut self,
        rng: &mut R,
        user_id: &UserIdRef,
        text: &str,
    ) -> Result<AIResponse>
    where
        R: Rng,
    {
        let start = Instant::now();
        let lock = self.state.read().await;
        let voice = match lock.voice_by_user_id(user_id) {
            Some(voice) => voice,
            None => Sample::random(rng)?,
        };
        debug!("running prompts with {voice}: {text}");
        let res_text = self.llm.prompt(text)?;
        debug!("response text is: {res_text}");
        let (req, res) = self.tts.prompt(voice, &res_text)?;
        debug!("complete. took {}s", start.elapsed().as_secs());
        Ok(AIResponse::new(req, res_text, res))
    }
}

#[derive(Debug)]
pub struct AIResponse {
    pub request_wav: AudioBuffer,
    pub response_text: String,
    pub response_wav: AudioBuffer,
}

impl AIResponse {
    fn new(request_wav: AudioBuffer, response_text: String, response_wav: AudioBuffer) -> Self {
        Self {
            request_wav,
            response_text,
            response_wav,
        }
    }
}
