use crate::state::AppState;
use crate::websocket::connection::WebsocketConnection;
use crate::websocket::session::WebsocketSession;
use anyhow::{Result, ensure};
use std::time::{Duration, Instant};
use tokio::spawn;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::{debug, warn};
use twitch_api::eventsub::Event;

mod connection;
mod session;

#[derive(Debug)]
pub struct WebsocketClient {
    receiver: Receiver<Event>,
    handle: Option<JoinHandle<Result<()>>>,
}

impl WebsocketClient {
    pub async fn start(state: AppState) -> Result<WebsocketClient> {
        ensure!(
            state.read().await.user_token().is_some(),
            "user must be logged in"
        );
        let (session, receiver) = WebsocketSession::new();
        let handle = WebsocketClientThread::start(state, session);
        Ok(Self {
            receiver,
            handle: Some(handle),
        })
    }

    pub async fn recv(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }

    pub async fn join(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.await??;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct WebsocketClientThread {
    state: AppState,
    session: WebsocketSession,
}

impl WebsocketClientThread {
    fn start(state: AppState, session: WebsocketSession) -> JoinHandle<Result<()>> {
        let thread = Self { state, session };
        spawn(Self::run(thread))
    }

    async fn run(self) -> Result<()> {
        loop {
            let mut conn = WebsocketConnection::start(self.state.clone(), self.session.clone());
            while conn.is_running() {
                let now = Instant::now();
                let last_seen = self.session.last_seen().await;
                let elapsed = now - last_seen;
                debug!("keep alive check. elapsed: {}s", elapsed.as_secs());
                if elapsed > Duration::from_mins(1) {
                    warn!("keep alive timeout. restarting connection");
                    conn.abort();
                    break;
                }
                sleep(Duration::from_secs(20)).await;
            }
            if let Err(e) = conn.join().await {
                warn!("websocket connection closed with error: {e}");
            }
        }
    }
}
