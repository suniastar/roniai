use anyhow::Result;
use tokio::sync::broadcast::{Receiver, Sender, channel};
use tracing::warn;

mod persistent;
#[derive(Debug)]
pub struct AppState {
    broadcast_sender: Sender<u64>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = channel(8);
        Self {
            broadcast_sender: tx,
        }
    }

    pub fn send(&mut self, value: u64) -> usize {
        self.broadcast_sender.send(value).unwrap_or_else(|e| {
            warn!("no receivers subscribed: {e}");
            0
        })
    }

    pub fn subscribe(&self) -> Receiver<u64> {
        self.broadcast_sender.subscribe()
    }
}
