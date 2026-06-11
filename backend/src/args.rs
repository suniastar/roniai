use clap::Parser;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::path::PathBuf;
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, env, help = "The twitch client id", required = true)]
    twitch_client_id: String,

    #[arg(long, env, help = "The twitch client secret", required = true)]
    twitch_client_secret: String,

    #[arg(long, env, help = "The applications log level", default_value_t = LevelFilter::INFO)]
    log_level: LevelFilter,

    #[arg(
        long,
        env,
        help = "Persistent storage for ai models and configs",
        default_value = "persistent.json"
    )]
    persistent: PathBuf,

    #[arg(
        long,
        env,
        help = "The webserver and websockets port (must use 80, 3000 or 8080 on first login)",
        default_value_t = 8080
    )]
    port: u16,

    #[arg(long, env, help = "The GPU id the models should run on")]
    gpu: Option<usize>,

    #[arg(
        long,
        env,
        help = "The huggingface repo of the llm",
        default_value = "mradermacher/MN-Violet-Lotus-12B-GGUF"
    )]
    llm_repo: String,

    #[arg(
        long,
        env,
        help = "The huggingface llm model file within the repo",
        default_value = "MN-Violet-Lotus-12B.Q4_K_M.gguf"
    )]
    llm_file: String,

    #[arg(long, env, help = "The LLM's temperature", default_value_t = 0.9)]
    llm_temp: f64,

    #[arg(long, env, help = "The LLM's top p", default_value = "1")]
    llm_top_p: Option<f64>,

    #[arg(long, env, help = "The LLM's top k")]
    llm_top_k: Option<usize>,

    #[arg(long, env, help = "The LLM's min p", default_value_t = 0.05)]
    llm_min_p: f64,

    #[arg(long, env, help = "The LLM's penalty window length")]
    llm_penalty_length: Option<usize>,

    #[arg(
        long,
        env,
        help = "The LLM's repetition penalty",
        default_value_t = 1.05
    )]
    llm_penalty_repeat: f32,

    #[arg(long, env, help = "The LLM's frequency penalty", default_value_t = 0.0)]
    llm_penalty_freq: f32,

    #[arg(long, env, help = "The LLM's preset penalty", default_value_t = 0.0)]
    llm_penalty_present: f32,

    #[arg(
        long,
        env,
        help = "The LLM's sampler starting seed",
        default_value_t = 42
    )]
    llm_seed: u32,

    #[arg(
        long,
        env,
        help = "When enabled use Qwen3 TTS 0.6B instead of the more accurate 1.7B",
        default_value_t = false
    )]
    tts_low_quality: bool,
}

impl Args {
    pub fn twitch_client_id(&self) -> &str {
        &self.twitch_client_id
    }

    pub fn twitch_client_secret(&self) -> &str {
        &self.twitch_client_secret
    }

    pub fn log_level(&self) -> LevelFilter {
        self.log_level
    }

    pub fn persistent(&self) -> &PathBuf {
        &self.persistent
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn gpu(&self) -> Option<usize> {
        self.gpu
    }

    pub fn llm_repo(&self) -> &str {
        &self.llm_repo
    }

    pub fn llm_file(&self) -> &str {
        &self.llm_file
    }

    pub fn llm_temp(&self) -> f64 {
        self.llm_temp
    }

    pub fn llm_top_p(&self) -> Option<f64> {
        self.llm_top_p
    }

    pub fn llm_top_k(&self) -> Option<usize> {
        self.llm_top_k
    }

    pub fn llm_min_p(&self) -> f64 {
        self.llm_min_p
    }

    pub fn llm_penalty_length(&self) -> Option<usize> {
        self.llm_penalty_length
    }

    pub fn llm_penalty_repeat(&self) -> f32 {
        self.llm_penalty_repeat
    }

    pub fn llm_penalty_freq(&self) -> f32 {
        self.llm_penalty_freq
    }

    pub fn llm_penalty_present(&self) -> f32 {
        self.llm_penalty_present
    }

    pub fn llm_seed(&self) -> u32 {
        self.llm_seed
    }

    pub fn tts_low_quality(&self) -> bool {
        self.tts_low_quality
    }

    #[cfg(test)]
    pub fn sample_args_with_low_tts() -> Self {
        Self {
            tts_low_quality: true,
            ..Self::sample()
        }
    }

    #[cfg(test)]
    pub fn sample_args_with_high_tts() -> Self {
        Self {
            tts_low_quality: false,
            ..Self::sample()
        }
    }

    #[cfg(test)]
    fn sample() -> Self {
        Self {
            twitch_client_id: "".to_string(),
            twitch_client_secret: "".to_string(),
            log_level: LevelFilter::INFO,
            persistent: "./target/persistent.json".into(),
            port: 8080,
            gpu: None,
            llm_repo: "mradermacher/MN-Violet-Lotus-12B-GGUF".into(),
            llm_file: "MN-Violet-Lotus-12B.Q4_K_M.gguf".into(),
            llm_temp: 0.8,
            llm_top_p: Some(1.0),
            llm_top_k: None,
            llm_min_p: 0.0,
            llm_penalty_length: None,
            llm_penalty_repeat: 1.0,
            llm_penalty_freq: 0.0,
            llm_penalty_present: 1.5,
            llm_seed: 42,
            tts_low_quality: false,
        }
    }
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "--twitch-client-id {} --twitch-client-secret {} --log-level {} --persistent {} --port {} --gpu {} --llm-repo {} --llm-file {} --llm-temp {} --llm-top-p {} --llm-top-k {} --llm-min-p {} --llm-penalty-length {} --llm-peanalty-repeat {} --llm-penalty-freq {} --llm-penalty-present {} --llm-seed {} --tts-low-quality {}",
            self.twitch_client_id,
            self.twitch_client_secret,
            self.log_level,
            self.persistent.display(),
            self.port,
            self.gpu
                .as_ref()
                .map(<usize>::to_string)
                .unwrap_or(String::from("null")),
            self.llm_repo,
            self.llm_file,
            self.llm_temp,
            self.llm_top_p
                .as_ref()
                .map(<f64>::to_string)
                .unwrap_or(String::from("null")),
            self.llm_top_k
                .as_ref()
                .map(<usize>::to_string)
                .unwrap_or(String::from("null")),
            self.llm_min_p,
            self.llm_penalty_length
                .as_ref()
                .map(<usize>::to_string)
                .unwrap_or(String::from("null")),
            self.llm_penalty_repeat,
            self.llm_penalty_freq,
            self.llm_penalty_present,
            self.llm_seed,
            self.tts_low_quality,
        )
    }
}
