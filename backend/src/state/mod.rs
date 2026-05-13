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

    pub fn send(&mut self, value: u64) -> Result<usize> {
        if self.broadcast_sender.is_empty() {
            warn!("no receivers. skip send");
            return Ok(0);
        }
        let n_rx = self.broadcast_sender.send(value)?;
        Ok(n_rx)
    }

    pub fn subscribe(&self) -> Receiver<u64> {
        self.broadcast_sender.subscribe()
    }
}
