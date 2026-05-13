use crate::args::Args;
use crate::server::WebsocketServer;
use crate::state::AppState;
use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::Layer as FmtLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

mod args;
mod init;
mod server;
mod state;

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
    let state = Arc::new(RwLock::new(AppState::new()));

    let handle = {
        let s = state.clone();
        spawn(async move {
            let mut c = 0;
            loop {
                c += 1;
                let mut lock = s.write().await;
                let n_rx = lock.send(c);
                drop(lock);
                info!("send {c} to {n_rx} receivers");
                sleep(Duration::from_secs(1)).await;
            }
        })
    };

    let server = WebsocketServer::new(state, args.port());
    server.run().await?;
    handle.abort();
    handle.await?;
    Ok(())
}
