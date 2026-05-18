use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use twitch_api::eventsub::Event;

const TWITCH_WEBSOCKET_SERVER: &str =
    "wss://eventsub.wss.twitch.tv/ws?keepalive_timeout_seconds=10";

#[derive(Debug, Clone)]
pub struct WebsocketSession {
    inner: Arc<RwLock<WebsocketSessionInner>>,
}

#[derive(Debug)]
struct WebsocketSessionInner {
    last_seen: Instant,
    url: Option<String>,
    sender: Sender<Event>,
}

impl WebsocketSession {
    pub fn new() -> (Self, Receiver<Event>) {
        let (tx, rx) = channel::<Event>(128);
        let me = Self {
            inner: Arc::new(RwLock::new(WebsocketSessionInner {
                last_seen: Instant::now(),
                url: None,
                sender: tx,
            })),
        };
        (me, rx)
    }

    pub async fn tick_last_seen(&self) {
        let now = Instant::now();
        let mut lock = self.inner.write().await;
        lock.last_seen = now;
    }

    pub async fn last_seen(&self) -> Instant {
        let lock = self.inner.read().await;
        lock.last_seen
    }

    pub async fn set_url(&self, url: Option<impl Into<String>>) {
        let mut lock = self.inner.write().await;
        lock.url = url.map(Into::into);
    }

    pub async fn url(&self) -> String {
        let lock = self.inner.read().await;
        lock.url
            .as_deref()
            .unwrap_or(TWITCH_WEBSOCKET_SERVER)
            .to_owned()
    }

    pub async fn send(&self, event: Event) -> Result<()> {
        let lock = self.inner.read().await;
        lock.sender.send(event).await?;
        Ok(())
    }
}
