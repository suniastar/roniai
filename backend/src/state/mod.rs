use crate::args::Args;
use crate::twitch::TwitchClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_yaml::{from_reader, to_writer};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use twitch_api::twitch_oauth2::{ClientId, ClientSecret, RefreshToken};
use twitch_api::types::UserId;

pub type AppState = Arc<RwLock<AppStateInner>>;

#[derive(Debug)]
pub struct AppStateInner {
    path: PathBuf,
    twitch: TwitchClient,
    voice_by_user_id: HashMap<UserId, ()>,
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

        let persistent = from_reader::<&File, PersistentState>(&file)?;
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
        to_writer(file, &persistent)?;
        Ok(())
    }

    pub fn twitch(&mut self) -> &mut TwitchClient {
        &mut self.twitch
    }

    pub fn voice_by_user_id(&self, user_id: UserId) -> Option<&()> {
        self.voice_by_user_id.get(&user_id)
    }

    async fn new(args: &Args) -> Result<Self> {
        let path = args.storage().join("persistent.yaml");
        let client_id = ClientId::from_str(args.twitch_client_id())?;
        let client_secret = ClientSecret::from_str(args.twitch_client_secret())?;
        let twitch = TwitchClient::new(client_id, client_secret).await?;
        let voice_by_user_id = HashMap::new();
        Ok(Self {
            path,
            twitch,
            voice_by_user_id,
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistentState {
    refresh_token: Option<RefreshToken>,
    voice_by_user_id: HashMap<UserId, ()>,
}

impl PersistentState {
    async fn extend(self, state: &mut AppStateInner, path: PathBuf) -> Result<()> {
        state.path = path;
        if let Some(refresh_token) = self.refresh_token {
            state.twitch.set_user_token(refresh_token).await?;
        }
        state.voice_by_user_id = self.voice_by_user_id;
        Ok(())
    }
}

impl From<&AppStateInner> for PersistentState {
    fn from(value: &AppStateInner) -> Self {
        Self {
            refresh_token: value
                .twitch
                .user_token()
                .map(|t| t.refresh_token.clone())
                .flatten(),
            voice_by_user_id: value.voice_by_user_id.clone(),
        }
    }
}
