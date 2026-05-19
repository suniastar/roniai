use crate::state::AppState;
use crate::twitch::default_helix_client;
use crate::websocket::session::WebsocketSession;
use anyhow::{Context, Result, anyhow};
use futures_util::StreamExt;
use std::fmt::Debug;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tracing::{debug, error, info, warn};
use twitch_api::eventsub::channel::ChannelChatMessageV1;
use twitch_api::eventsub::{Event, EventsubWebsocketData, Transport};

#[derive(Debug)]
pub struct WebsocketConnection {
    handle: Option<JoinHandle<Result<()>>>,
}

impl WebsocketConnection {
    pub fn start(state: AppState, session: WebsocketSession) -> Self {
        let handle = WebsocketConnectionThread::start(state, session);
        Self {
            handle: Some(handle),
        }
    }

    pub fn is_running(&self) -> bool {
        match self.handle.as_ref() {
            None => false,
            Some(h) => !h.is_finished(),
        }
    }

    pub fn abort(&mut self) {
        if let Some(h) = self.handle.as_mut() {
            h.abort();
        }
    }

    pub async fn join(&mut self) -> Result<()> {
        if let Some(h) = self.handle.take() {
            match h.await {
                Err(e) => {
                    if !e.is_cancelled() {
                        return Err(e.into());
                    }
                }
                Ok(result) => result?,
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct WebsocketConnectionThread {
    state: AppState,
    session: WebsocketSession,
    run: bool,
}

impl WebsocketConnectionThread {
    fn start(state: AppState, session: WebsocketSession) -> JoinHandle<Result<()>> {
        let thread = Self {
            state,
            session,
            run: true,
        };
        spawn(Self::run(thread))
    }

    async fn run(mut self) -> Result<()> {
        // connect to eventsub
        let mut socket = self.connect().await?;

        // receive loop
        let mut result: Result<()> = Ok(());
        while self.run {
            let message = match socket.next().await {
                None => {
                    result = Err(anyhow!("websocket closed"));
                    break;
                }
                Some(Err(e)) => {
                    result = Err(e.into());
                    break;
                }
                Some(Ok(msg)) => msg,
            };
            self.handle_websocket_message(message).await;
        }

        // shutdown
        result
    }

    async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        let url = self.session.url().await;
        info!("connecting to twitch event sub: {url}");
        let (socket, _) = connect_async(&url).await?;
        self.session.tick_last_seen().await;
        Ok(socket)
    }

    async fn handle_websocket_message(&mut self, message: Message) {
        // update last seen / last online
        self.session.tick_last_seen().await;

        // handle message
        match message {
            Message::Text(text) => {
                let data = match Event::parse_websocket(&text) {
                    Err(e) => {
                        error!("failed to parse payload: {e}");
                        return;
                    }
                    Ok(event) => event,
                };
                self.handle_eventsub_data(data).await;
            }
            Message::Binary(_) => {
                warn!("unsupported websocket message type: binary");
            }
            Message::Ping(_) => {} // nothing to do
            Message::Pong(_) => {} // nothing to do
            Message::Close(close) => {
                match close {
                    Some(CloseFrame { code, reason, .. }) => {
                        debug!("connection closed from remote host: code={code}, reason={reason}")
                    }
                    None => debug!("connection closed from remote host: code=none, reason=none"),
                }
                self.run = false;
            }
            Message::Frame(_) => {
                warn!("unsupported websocket message type: frame");
            }
        }
    }

    async fn handle_eventsub_data(&mut self, data: EventsubWebsocketData<'_>) {
        match data {
            EventsubWebsocketData::Welcome { payload, .. } => {
                let session_id = payload.session.id.as_ref();
                match self.register_event_subscribers(session_id).await {
                    Err(e) => warn!("failed to register subscribers: {e}"),
                    Ok(()) => info!("connection established: {session_id}"),
                };
            }
            EventsubWebsocketData::Keepalive { .. } => {} // nothing to do
            EventsubWebsocketData::Notification { payload, .. } => {
                if let Err(e) = self.session.send(payload).await {
                    debug!("failed to send to pipe: {e}");
                    self.run = false;
                }
            }
            EventsubWebsocketData::Revocation { .. } => error!("revocation data: {data:?}"),
            EventsubWebsocketData::Reconnect { payload, .. } => {
                let url = payload.session.reconnect_url.as_deref();
                info!(
                    "twitch requested a client reconnect: {}",
                    url.unwrap_or("none")
                );
                self.session.set_url(url).await;
                self.run = false;
            }
            _ => error!("unknown event: {data:?}"),
        }
    }

    async fn register_event_subscribers(&self, session_id: &str) -> Result<()> {
        let helix = default_helix_client();
        let mut lock = self.state.write().await;
        let mut tried_refresh = false;
        loop {
            let user_token = lock.user_token().context("user is not logged in")?;
            let broadcaster_id = user_token.user_id.clone();
            let bot_id = user_token.user_id.clone();
            let event = ChannelChatMessageV1::new(broadcaster_id, bot_id);
            let transport = Transport::websocket(session_id);
            match helix
                .create_eventsub_subscription(event, transport, user_token)
                .await
            {
                Err(e) => {
                    if !tried_refresh {
                        warn!("request failed and will be tried again after refreshing token: {e}");
                        lock.refresh_app_token(&helix).await?;
                        lock.save().await?;
                        tried_refresh = true;
                    } else {
                        return Err(e)?;
                    }
                }
                Ok(_) => return Ok(()),
            }
        }
    }
}
