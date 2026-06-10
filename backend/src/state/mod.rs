use crate::args::Args;
use crate::twitch::{TWITCH_BOT_SCOPES, default_helix_client};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader as json_deserialize, to_writer as json_serialize};
use std::fs::{File, OpenOptions};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use twitch_api::HelixClient;
use twitch_api::twitch_oauth2::{
    AppAccessToken, ClientId, ClientIdRef, ClientSecret, ClientSecretRef, RefreshToken, UserToken,
};
use twitch_api::types::RewardId;

pub type AppState = Arc<RwLock<AppStateInner>>;

#[derive(Debug)]
pub struct AppStateInner {
    path: PathBuf,
    client_id: ClientId,
    client_secret: ClientSecret,
    app_token: AppAccessToken,
    user_token: Option<UserToken>,
    reward_id: Option<RewardId>,
}

impl AppStateInner {
    pub async fn load(args: &Args) -> Result<AppState> {
        let mut state = Self::new(args).await?;

        if !state.path.try_exists()? {
            warn!("config not found. starting blank");
            state.save().await?;
            state.path = state.path.canonicalize()?;
            return Ok(Arc::new(RwLock::new(state)));
        }

        let abs = state.path.canonicalize()?;
        info!("reading config from {}", abs.display());
        let file = OpenOptions::new()
            .create(false)
            .truncate(false)
            .read(true)
            .write(false)
            .open(&abs)?;
        let persistent = json_deserialize::<&File, PersistentState>(&file)?;
        persistent.extend(&mut state, abs).await?;

        Ok(Arc::new(RwLock::new(state)))
    }

    pub async fn save(&self) -> Result<()> {
        info!("Saving config to {}", self.path.display());

        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(false)
            .write(true)
            .open(&self.path)?;

        #[cfg(unix)] // set permissions on unix systems
        {
            let mut permissions = file.metadata()?.permissions();
            if permissions.mode() != 0o600 {
                permissions.set_mode(0o600);
            }
            file.set_permissions(permissions)?;
        }

        let persistent = PersistentState::from(self);
        json_serialize(file, &persistent)?;
        Ok(())
    }

    pub fn client_id(&self) -> &ClientIdRef {
        self.client_id.as_ref()
    }

    pub fn client_secret(&self) -> &ClientSecretRef {
        self.client_secret.as_ref()
    }

    #[allow(unused)]
    pub fn app_token(&self) -> &AppAccessToken {
        &self.app_token
    }

    pub fn set_user_token(&mut self, user_token: UserToken) {
        self.user_token = Some(user_token);
    }

    pub fn user_token(&self) -> Option<&UserToken> {
        self.user_token.as_ref()
    }

    pub fn mut_user_token(&mut self) -> &mut Option<UserToken> {
        &mut self.user_token
    }

    pub fn reward_id(&self) -> Option<&RewardId> {
        self.reward_id.as_ref()
    }

    pub async fn refresh_app_token(&mut self, helix: &HelixClient<'static, Client>) -> Result<()> {
        let app_token = AppAccessToken::get_app_access_token(
            helix.get_client(),
            self.client_id.clone(),
            self.client_secret.clone(),
            TWITCH_BOT_SCOPES.to_vec(),
        )
        .await?;
        self.app_token = app_token;
        Ok(())
    }

    async fn new(args: &Args) -> Result<Self> {
        let path = args.persistent().to_owned();
        let client_id = ClientId::from_str(args.twitch_client_id())?;
        let client_secret = ClientSecret::from_str(args.twitch_client_secret())?;
        let helix = default_helix_client();
        let app_token = AppAccessToken::get_app_access_token(
            helix.get_client(),
            client_id.clone(),
            client_secret.clone(),
            TWITCH_BOT_SCOPES.to_vec(),
        )
        .await?;
        Ok(Self {
            path,
            client_id,
            client_secret,
            app_token,
            user_token: None,
            reward_id: None,
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistentState {
    refresh_token: Option<RefreshToken>,
    reward_id: Option<RewardId>,
}

impl PersistentState {
    async fn extend(self, state: &mut AppStateInner, path: PathBuf) -> Result<()> {
        state.path = path;
        if let Some(refresh_token) = self.refresh_token {
            let helix = default_helix_client();
            let token = UserToken::from_refresh_token(
                helix.get_client(),
                refresh_token,
                state.client_id.clone(),
                state.client_secret.clone(),
            )
            .await?;
            state.user_token = Some(token);
        }
        state.reward_id = self.reward_id;
        Ok(())
    }
}

impl From<&AppStateInner> for PersistentState {
    fn from(value: &AppStateInner) -> Self {
        Self {
            refresh_token: value
                .user_token
                .as_ref()
                .and_then(|t| t.refresh_token.clone()),
            reward_id: value.reward_id.clone(),
        }
    }
}
