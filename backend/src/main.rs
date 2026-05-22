use crate::ai::AI;
use crate::args::Args;
use crate::server::Server;
use crate::state::AppStateInner;
use crate::websocket::WebsocketClient;
use anyhow::Result;
use clap::Parser;
use rand::{Rng, rng};
use tracing::{debug, error, info, warn};
use tracing_subscriber::fmt::Layer as FmtLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use twitch_api::eventsub::{Event, Message};

mod ai;
mod args;
mod init;
mod server;
mod state;
mod twitch;
mod websocket;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
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
                        .with_default_directive(args.log_level().into())
                        .from_env_lossy(),
                ),
        )
        .try_init()?;
    info!("starting with: {}", args);
    debug!("debug args: {args:?}");

    let state = AppStateInner::load(&args).await?;
    let mut ai = AI::new(&args, state.clone())?;

    let mut client = match WebsocketClient::start(state.clone()).await {
        Err(e) => {
            warn!("running without websocket client: {e}");
            None
        }
        Ok(client) => Some(client),
    };
    let mut server = Server::start(state.clone(), args.port());

    if let Some(c) = client.as_mut() {
        let mut rng = rng();
        loop {
            match c.recv().await {
                None => break,
                Some(event) => match event {
                    Event::ChannelChatMessageV1(payload) => match payload.message {
                        Message::Notification(data) => {
                            let id = rng.next_u64();
                            let text = format!("Hey Roni AI. {}", data.message.text);
                            let user_id = data.chatter_user_id;
                            let n1 = server.send_eval(id, text.clone());
                            info!("send eval \"{text}\" to {n1} clients");
                            let res = ai.eval(&mut rng, &user_id, &text).await?;
                            let n2 = server.send_say(id, res);
                            info!("send say to {n2} clients");
                        }
                        _ => {
                            error!("unknown message: {:?}", payload);
                        }
                    },
                    _ => {
                        error!("unknown event: {:?}", event);
                    }
                },
            }
        }
    }

    server.join().await?;
    if let Some(c) = client.as_mut() {
        c.join().await?;
    }
    Ok(())
}
