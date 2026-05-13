use crate::server::endpoints::ws_mrsroni;
use crate::state::AppState;
use anyhow::Result;
use axum::routing::get;
use axum::{Router, serve};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;

mod endpoints;

#[derive(Debug)]
pub struct WebsocketServer {
    state: Arc<RwLock<AppState>>,
    port: u16,
}

impl WebsocketServer {
    pub fn new(state: Arc<RwLock<AppState>>, port: u16) -> WebsocketServer {
        Self { state, port }
    }

    pub async fn run(&self) -> Result<()> {
        let router = Router::new()
            .route("/ws/mrsroni", get(ws_mrsroni))
            .with_state(self.state.clone());
        info!("Binding listeners...");
        let listener = TcpListener::bind(format!("[::]:{}", self.port)).await?;

        info!("Running webserver on {}", self.port);
        serve(listener, router).await?;

        Ok(())
    }
}
