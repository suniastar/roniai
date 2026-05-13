use crate::state::AppState;
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

pub async fn ws_mrsroni(
    State(state): State<Arc<RwLock<AppState>>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_failed_upgrade(|e| warn!("failed to upgrade websocket: {}", e))
        .on_upgrade(|ws| ws_mrsroni_handle(state, ws))
}

async fn ws_mrsroni_handle(state: Arc<RwLock<AppState>>, mut socket: WebSocket) {
    let lock = state.read().await;
    let mut broadcast = lock.subscribe();
    drop(lock);

    loop {
        let message = match broadcast.recv().await {
            Err(e) => {
                warn!("failed to receive message via broadcast: {e}");
                break;
            }
            Ok(msg) => msg,
        };
        let text = format!("Send Message Nr. {message}");
        if let Err(e) = socket.send(Message::Text(text.into())).await {
            warn!("failed to send message via websocket: {e}");
        }
    }

    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: 1000,
            reason: Utf8Bytes::from_static("bye"),
        })))
        .await;
}
