use crate::ai::llm::LLM;
use anyhow::Result;

mod llm;

pub struct AI {
    llm: LLM,
}

impl AI {
    pub fn new() -> Result<Self> {
        let llm = LLM::new()?;
        Ok(Self { llm })
    }

    pub fn eval(&mut self, text: &str) -> Result<String> {
        self.llm.prompt(text)
    }
}
