use axum::http::HeaderValue;
use reqwest::Client;
use twitch_api::HelixClient;
use twitch_api::client::ClientDefault;
use twitch_api::helix::Scope;

pub const TWITCH_USER_SCOPES: [Scope; 2] = [Scope::ChannelBot, Scope::UserReadChat];

pub const TWITCH_BOT_SCOPES: [Scope; 1] = [Scope::ChannelBot];

pub fn default_helix_client() -> HelixClient<'static, Client> {
    const CLIENT_PRODUCT: HeaderValue = HeaderValue::from_static(env!("CARGO_PKG_NAME"));
    let client = Client::default_client_with_name(Some(CLIENT_PRODUCT))
        .expect("initializing this static helix client should never fail");
    HelixClient::with_client(client)
}
