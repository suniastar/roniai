use serde::{Deserialize, Serialize};
use twitch_api::twitch_oauth2::RefreshToken;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PersistentState {
    twitch_refresh_token: Option<RefreshToken>,
}
