use crate::ai::llm::LLM;
use crate::args::Args;
use anyhow::Result;

mod llm;

pub struct AI {
    llm: LLM,
}

impl AI {
    pub fn new(args: &Args) -> Result<Self> {
        let llm = LLM::new(args)?;
        Ok(Self { llm })
    }

    pub fn eval(&mut self, text: &str) -> Result<String> {
        self.llm.prompt(text)
    }
}
