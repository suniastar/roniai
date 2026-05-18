use crate::args::Args;
use crate::server::Server;
use crate::state::{AppState, AppStateInner};
use crate::websocket::WebsocketClient;
use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::Layer as FmtLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use twitch_api::eventsub::{Event, Message};

mod args;
mod init;
mod server;
mod state;
mod twitch;
mod websocket;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            FmtLayer::new()
                .compact()
                .with_level(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_target(true)
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .from_env_lossy(),
                ),
        )
        .try_init()?;

    let args = Args::parse();
    let state = AppStateInner::load(&args).await?;

    // let mut client = WebsocketClient::start();
    let mut server = Server::start(state, args.port());

    server.join().await?;
    // client.join().await?;
    Ok(())
}
