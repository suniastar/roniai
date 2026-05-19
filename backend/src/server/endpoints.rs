use crate::server::session::{LoginSession, ServerSession};
use crate::twitch::{TWITCH_USER_SCOPES, default_helix_client};
use anyhow::{Context, Result, bail};
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect};
use serde::Deserialize;
use std::collections::HashSet;
use tracing::warn;
use twitch_api::helix::Scope;
use twitch_api::twitch_oauth2::{CsrfToken, UserToken};

pub async fn twitch_login(State(session): State<LoginSession>) -> impl IntoResponse {
    let state = session.state().await;
    let lock = state.read().await;
    let callback_url = session.callback_url().await;
    let (url, csrf) = UserToken::builder(
        lock.client_id().to_owned(),
        lock.client_secret().to_owned(),
        callback_url,
    )
    .set_scopes(TWITCH_USER_SCOPES.to_vec())
    .force_verify(true)
    .generate_url();
    session.set_scopes(&TWITCH_USER_SCOPES).await;
    session.set_csrf_token(csrf.clone()).await;
    Redirect::temporary(url.as_str())
}

#[derive(Debug, Deserialize)]
pub(crate) struct CallbackParams {
    state: String,
    #[serde(flatten)]
    data: CallbackParamData,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CallbackParamData {
    Ok {
        #[serde(rename = "scope")]
        scopes: String,
        #[serde(rename = "code")]
        code: String,
    },
    Error {
        #[serde(rename = "error")]
        name: String,
        #[serde(rename = "error_description")]
        description: String,
    },
}

pub async fn twitch_callback(
    State(session): State<LoginSession>,
    Query(params): Query<CallbackParams>,
) -> impl IntoResponse {
    match verify_callback_and_save_token(session, params).await {
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                r#"
<!DOCTYPE html>
<html lang="en">
    <title>
    </title>
    <body>
        <h1>Error</h1>
        <p>{e}</p>
    </body>
</html>
            "#
            )),
        ),
        Ok(()) => (
            StatusCode::OK,
            Html(
                r#"
<!DOCTYPE html>
<html lang="en">
    <title>
    </title>
    <body>
        <h1>Success</h1>
        <p>You can close this page now.</p>
    </body>
</html>
            "#
                .into(),
            ),
        ),
    }
}

async fn verify_callback_and_save_token(
    session: LoginSession,
    params: CallbackParams,
) -> Result<()> {
    // check csrf
    let param_csrf = CsrfToken::from(params.state);
    let session_csrf = match session.csrf_token().await {
        None => bail!("no csrf token. call login first."),
        Some(csrf) => csrf,
    };
    if param_csrf != session_csrf {
        bail!("CSRF token mismatch");
    }

    // check if user accepted
    let (param_scopes, param_code) = match params.data {
        CallbackParamData::Error { name, description } => bail!("api error: {name}: {description}"),
        CallbackParamData::Ok { scopes, code } => (scopes, code),
    };

    // check scopes
    let state_scopes = session
        .scopes()
        .await
        .context("missing scopes. call login first")?;
    let scopes_expected = state_scopes
        .iter()
        .map(Scope::as_static_str)
        .collect::<HashSet<&'static str>>();
    let scopes_received = param_scopes.split(" ").collect::<HashSet<&str>>();
    if scopes_expected != scopes_received {
        bail!("scope mismatch");
    }

    // request token (code flow)staate_scope
    let helix = default_helix_client();
    let state = session.state().await;
    let mut lock = state.write().await;
    let callback_url = session.callback_url().await;
    let mut builder = UserToken::builder(
        lock.client_id().to_owned(),
        lock.client_secret().to_owned(),
        callback_url,
    )
    .set_scopes(state_scopes.to_vec())
    .force_verify(true);
    builder.set_csrf(param_csrf);
    let token = builder
        .get_user_token(helix.get_client(), session_csrf.as_str(), &param_code)
        .await?;
    lock.set_user_token(token);
    lock.save().await?;
    Ok(())
}

pub async fn ws_mrsroni(
    State(session): State<ServerSession>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_failed_upgrade(|e| warn!("failed to upgrade websocket: {}", e))
        .on_upgrade(|ws| ws_mrsroni_handle(session, ws))
}

async fn ws_mrsroni_handle(session: ServerSession, mut socket: WebSocket) {
    let mut broadcast = session.subscribe();
    loop {
        let message = match broadcast.recv().await {
            Err(e) => {
                warn!("failed to receive message via broadcast: {e}");
                break;
            }
            Ok(msg) => msg,
        };
        let msg = match Message::try_from(&message) {
            Err(e) => {
                warn!("failed to parse message: {e}");
                continue;
            }
            Ok(m) => m,
        };
        if let Err(e) = socket.send(msg).await {
            warn!("failed to send message via websocket: {e}");
            break;
        }
    }
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: 1000,
            reason: Utf8Bytes::from_static("bye"),
        })))
        .await;
}
