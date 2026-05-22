use clap::Parser;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::path::PathBuf;
use tracing_subscriber::filter::LevelFilter;

const DEFAULT_SYSTEM_MESSAGE: &str = r#"
Du bist Roni AI.
Eine künstliche Intelligenz, welche sich genau wie die echte Roni (auch MrsRoni, Bella oder Tonne genannt) verhalten soll.
Roni, und dadurch auch du, bist ein streamer, welcher hauptsächlich GTA RP auf Narco City spielt.
Ab und zu, spielst du mit Freunden und Zuschauern aber auch VALORANT, League of Legends und andere Spiele.
Du antwortest in einem passiven aggressiven Tonfall und versuchst dabei, deine Zuschauer ein bisschen zu roasten.
Wenn nicht anders angegeben, antwortest du nur auf Deutsch, aber du kannst Wörter aus anderen Sprachen verwenden, wenn sie passen.
Versuche, deine Antwort so kurz wie möglich und in einem menschenähnlichen Stil zu halten und vermeide Punktlisten und Emojis.
"#;

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

    #[arg(
        long,
        env,
        help = "The huggingface repo of the llm",
        default_value = "unsloth/Qwen3.5-9B-GGUF"
    )]
    llm_repo: String,

    #[arg(
        long,
        env,
        help = "The huggingface llm model file within the repo",
        default_value = "Qwen3.5-9B-Q8_0.gguf"
    )]
    llm_file: String,

    #[arg(
    long,
    env,
    help="The LLM's system message infront of every query.",
    default_value = DEFAULT_SYSTEM_MESSAGE,
    )]
    llm_system_message: String,

    #[arg(long, env, help = "The LLM's temperature", default_value_t = 1.0)]
    llm_temp: f32,

    #[arg(long, env, help = "The LLM's top p", default_value_t = 0.95)]
    llm_top_p: f32,

    #[arg(long, env, help = "The LLM's top k", default_value_t = 20)]
    llm_top_k: i32,

    #[arg(long, env, help = "The LLM's min p", default_value_t = 0.0)]
    llm_min_p: f32,

    #[arg(long, env, help = "The LLM's penalty window length", default_value_t = -1)]
    llm_penalty_length: i32,

    #[arg(
        long,
        env,
        help = "The LLM's repetition penalty",
        default_value_t = 1.0
    )]
    llm_penalty_repeat: f32,

    #[arg(long, env, help = "The LLM's frequency penalty", default_value_t = 0.0)]
    llm_penalty_freq: f32,

    #[arg(long, env, help = "The LLM's preset penalty", default_value_t = 1.5)]
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

    pub fn llm_repo(&self) -> &str {
        &self.llm_repo
    }

    pub fn llm_file(&self) -> &str {
        &self.llm_file
    }

    pub fn llm_system_message(&self) -> &str {
        &self.llm_system_message
    }

    pub fn llm_temp(&self) -> f32 {
        self.llm_temp
    }

    pub fn llm_top_p(&self) -> f32 {
        self.llm_top_p
    }

    pub fn llm_top_k(&self) -> i32 {
        self.llm_top_k
    }

    pub fn llm_min_p(&self) -> f32 {
        self.llm_min_p
    }

    pub fn llm_penalty_length(&self) -> i32 {
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
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "--twitch-client-id {} --twitch-client-secret {} --log-level {} --persistent {} --port {} --llm-repo {} --llm-file {} --llm-system-message [redacted] --llm-temp {} --llm-top-p {} --llm-top-k {} --llm-min-p {} --llm-penalty-length {} --llm-peanalty-repeat {} --llm-penalty-freq {} --llm-penalty-present {} --llm-seed {} --tts-low-quality {}",
            self.twitch_client_id,
            self.twitch_client_secret,
            self.log_level,
            self.persistent.display(),
            self.port,
            self.llm_repo,
            self.llm_file,
            self.llm_temp,
            self.llm_top_p,
            self.llm_top_k,
            self.llm_min_p,
            self.llm_penalty_length,
            self.llm_penalty_repeat,
            self.llm_penalty_freq,
            self.llm_penalty_present,
            self.llm_seed,
            self.tts_low_quality,
        )
    }
}
