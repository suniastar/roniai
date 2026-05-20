use crate::ai::llm::LLM;
use crate::ai::tts::TTS;
use anyhow::Result;

mod llm;
mod tts;

pub struct AI {
    llm: LLM,
    tts: TTS,
}

impl AI {
    pub fn new() -> Result<Self> {
        let llm = LLM::new()?;
        let tts = TTS::new()?;
        Ok(Self { llm, tts })
    }

    pub fn eval(&mut self, text: &str) -> Result<String> {
        let res_text = self.llm.prompt(text)?;
        let req = self.tts.prompt(text)?;
        let res = self.tts.prompt(&res_text)?;
        req.save("req.wav")?;
        req.save("res.wav")?;
        Ok(res_text)
    }
}
