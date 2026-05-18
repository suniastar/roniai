use anyhow::Result;
use axum::http::HeaderValue;
use reqwest::{Client, Url};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use twitch_api::HelixClient;
use twitch_api::client::ClientDefault;
use twitch_api::helix::Scope;
use twitch_api::twitch_oauth2::{
    AppAccessToken, ClientId, ClientSecret, CsrfToken, RefreshToken, UserToken,
};

const TWITCH_USER_SCOPES: [Scope; 1] = [Scope::ChannelBot];

const TWITCH_BOT_SCOPES: [Scope; 0] = [];

pub struct TwitchClient {
    client_id: ClientId,
    client_secret: ClientSecret,
    helix: HelixClient<'static, Client>,
    app_token: AppAccessToken,
    user_token: Option<UserToken>,
}

impl TwitchClient {
    pub fn user_scopes() -> &'static [Scope] {
        &TWITCH_USER_SCOPES
    }

    pub async fn new(client_id: ClientId, client_secret: ClientSecret) -> Result<Self> {
        const CLIENT_PRODUCT: HeaderValue = HeaderValue::from_static(env!("CARGO_PKG_NAME"));
        let client = Client::default_client_with_name(Some(CLIENT_PRODUCT))?;
        let helix = HelixClient::with_client(client);
        let app_token = AppAccessToken::get_app_access_token(
            helix.get_client(),
            client_id.clone(),
            client_secret.clone(),
            TWITCH_BOT_SCOPES.to_vec(),
        )
        .await?;
        Ok(Self {
            client_id,
            client_secret,
            helix,
            app_token,
            user_token: None,
        })
    }

    pub fn generate_login_url(&mut self, scopes: &[Scope], callback_url: Url) -> (Url, CsrfToken) {
        UserToken::builder(
            self.client_id.clone(),
            self.client_secret.clone(),
            callback_url,
        )
        .set_scopes(scopes.to_vec())
        .force_verify(true)
        .generate_url()
    }

    pub async fn set_user_token_from_response(
        &mut self,
        scopes: &[Scope],
        callback_url: Url,
        csrf: CsrfToken,
        code: String,
    ) -> Result<()> {
        let mut builder = UserToken::builder(
            self.client_id.clone(),
            self.client_secret.clone(),
            callback_url,
        )
        .set_scopes(scopes.to_vec())
        .force_verify(true);
        builder.set_csrf(csrf.clone());
        let token = builder
            .get_user_token(self.helix.get_client(), csrf.as_str(), &code)
            .await?;
        self.user_token = Some(token);
        Ok(())
    }

    pub async fn set_user_token(&mut self, refresh_token: RefreshToken) -> Result<()> {
        let token = UserToken::from_refresh_token(
            self.helix.get_client(),
            refresh_token,
            self.client_id.clone(),
            self.client_secret.clone(),
        )
        .await?;
        self.user_token = Some(token);
        Ok(())
    }

    pub fn user_token(&self) -> Option<&UserToken> {
        self.user_token.as_ref()
    }
}

impl Debug for TwitchClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("TwitchClient")
            .field("app_token", &"[redacted]")
            .finish_non_exhaustive()
    }
}
