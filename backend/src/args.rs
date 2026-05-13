use clap::Parser;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long, env, help = "The twitch client id", required = true)]
    twitch_client_id: String,

    #[arg(long, env, help = "The twitch client secret", required = true)]
    twitch_client_secret: String,

    #[arg(
        long,
        env,
        help = "Persistent storage for ai models and configs",
        required = true
    )]
    storage: PathBuf,

    #[arg(
        long,
        env,
        help = "The webserver and websockets port",
        default_value_t = 8080
    )]
    port: u16,
}

impl Args {
    pub fn twitch_client_id(&self) -> &str {
        &self.twitch_client_id
    }

    pub fn twitch_client_secret(&self) -> &str {
        &self.twitch_client_secret
    }

    pub fn storage(&self) -> &PathBuf {
        &self.storage
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Display for Args {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "--twitch-client-id {} --twitch-client-secret {} --storage {} --port {}",
            self.twitch_client_id,
            self.twitch_client_secret,
            self.storage.display(),
            self.port
        )
    }
}
