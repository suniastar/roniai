use crate::server::message::Message;
use crate::state::AppState;
use reqwest::Url;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast::{Receiver, Sender, channel};
use tracing::warn;
use twitch_api::helix::Scope;
use twitch_api::twitch_oauth2::CsrfToken;

#[derive(Debug, Clone)]
pub struct LoginSession {
    inner: Arc<RwLock<LoginSessionInner>>,
}

#[derive(Debug)]
pub struct LoginSessionInner {
    state: AppState,
    callback_url: Url,
    scopes: Option<&'static [Scope]>,
    csrf_token: Option<CsrfToken>,
}

impl LoginSession {
    pub fn new(state: AppState, callback_url: Url) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LoginSessionInner {
                state,
                callback_url,
                scopes: None,
                csrf_token: None,
            })),
        }
    }

    pub async fn state(&self) -> AppState {
        let lock = self.inner.read().await;
        lock.state.clone()
    }

    pub async fn callback_url(&self) -> Url {
        let lock = self.inner.read().await;
        lock.callback_url.clone()
    }

    pub async fn set_scopes(&self, scopes: &'static [Scope]) {
        let mut lock = self.inner.write().await;
        lock.scopes = Some(scopes);
    }

    pub async fn scopes(&self) -> Option<&'static [Scope]> {
        let lock = self.inner.read().await;
        lock.scopes
    }

    pub async fn set_csrf_token(&self, token: CsrfToken) {
        let mut lock = self.inner.write().await;
        lock.csrf_token = Some(token);
    }

    pub async fn csrf_token(&self) -> Option<CsrfToken> {
        let lock = self.inner.read().await;
        lock.csrf_token.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ServerSession {
    inner: Arc<ServerSessionInner>,
}

#[derive(Debug)]
pub struct ServerSessionInner {
    sender: Sender<Message>,
}

impl ServerSession {
    pub fn new() -> Self {
        let (tx, _) = channel(8);
        Self {
            inner: Arc::new(ServerSessionInner { sender: tx }),
        }
    }

    pub fn send(&self, value: Message) -> usize {
        self.inner.sender.send(value).unwrap_or_else(|e| {
            warn!("no receivers subscribed: {e}");
            0
        })
    }

    pub fn subscribe(&self) -> Receiver<Message> {
        self.inner.sender.subscribe()
    }
}
