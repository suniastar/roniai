use crate::server::endpoints::{twitch_callback, twitch_login, ws_mrsroni};
use crate::server::message::Message;
use crate::server::session::{LoginSession, ServerSession};
use crate::state::AppState;
use anyhow::Result;
use axum::routing::get;
use axum::{Router, serve};
use reqwest::Url;
use tokio::net::TcpListener;
use tokio::spawn;
use tokio::task::JoinHandle;
use tracing::info;

mod endpoints;
mod message;
mod session;

#[derive(Debug)]
pub struct Server {
    session: ServerSession,
    handle: Option<JoinHandle<Result<()>>>,
}

impl Server {
    pub fn start(state: AppState, port: u16) -> Server {
        let session = ServerSession::new();
        let handle = ServerThread::start(state, session.clone(), port);
        Self {
            session,
            handle: Some(handle),
        }
    }

    pub fn send_eval(&self, id: u64, message: String) -> usize {
        self.session.send(Message::EvaluatingPrompt {
            id,
            prompt: message,
        })
    }

    pub async fn join(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.await??;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ServerThread {
    login_session: LoginSession,
    server_session: ServerSession,
    port: u16,
}

impl ServerThread {
    fn start(state: AppState, session: ServerSession, port: u16) -> JoinHandle<Result<()>> {
        let callback_url = format!("http://localhost:{port}/twitch/callback");
        let url = Url::parse(&callback_url).expect("failed static url parse");
        let login_session = LoginSession::new(state, url);
        let thread = Self {
            login_session,
            server_session: session,
            port,
        };
        spawn(Self::run(thread))
    }

    async fn run(self) -> Result<()> {
        let router = Router::new()
            .nest(
                "/twitch",
                Router::new()
                    .route("/login", get(twitch_login))
                    .route("/callback", get(twitch_callback))
                    .with_state(self.login_session),
            )
            .nest(
                "/ws",
                Router::new()
                    .route("/mrsroni", get(ws_mrsroni))
                    .with_state(self.server_session),
            );
        info!("Binding listeners...");
        let listener = TcpListener::bind(format!("[::]:{}", self.port)).await?;

        info!("Running webserver on {}", self.port);
        serve(listener, router).await?;

        Ok(())
    }
}
