use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use chrono::{DateTime, Utc};
use librespot::core::{
    authentication::Credentials, cache::Cache, config::SessionConfig, keymaster, session::Session,
};
use rspotify::AuthCodeSpotify;
use tokio::sync::RwLock;
use url::Url;

use crate::{common::crypto, errors::ServerError};

pub mod utils;

struct Expiration {
    expires_in: i64,
    token_expiration: DateTime<Utc>,
}

impl Expiration {
    fn update_expires_in(&mut self, seconds: i64) {
        self.expires_in = seconds;
        self.token_expiration = Utc::now();
    }
}

impl Default for Expiration {
    fn default() -> Self {
        Expiration {
            expires_in: 0,
            token_expiration: Utc::now(),
        }
    }
}

pub struct SpotifyAccount {
    pub credentials: Credentials,
    pub session: Session,
    pub client: AuthCodeSpotify,
    expiration: RwLock<Expiration>,
    // Secret key
    secret: [u8; 16],
}

impl SpotifyAccount {
    pub async fn new(
        credentials: Credentials,
        cache: Cache,
        proxy: Option<Url>,
    ) -> Result<Self, ServerError> {
        let mut config = SessionConfig::default();
        config.proxy = proxy;
        let session = Session::connect(config, credentials.clone(), Some(cache)).await?;
        let client: AuthCodeSpotify = AuthCodeSpotify::default();
        let secret: [u8; 16] = rand::random();
        let account = SpotifyAccount {
            credentials,
            session,
            client,
            expiration: RwLock::new(Expiration::default()),
            secret,
        };

        Ok(account)
    }

    pub async fn create<P>(
        credentials: Credentials,
        cache_dir: Option<P>,
        proxy: Option<Url>,
    ) -> Result<Self, ServerError>
    where
        P: AsRef<Path>,
    {
        let cache = Cache::new(cache_dir, None, None)?;
        SpotifyAccount::new(credentials, cache, proxy).await
    }

    async fn token_expires(&self) -> bool {
        let expiration = self.expiration.read().await;
        let delta = Utc::now() - expiration.token_expiration;
        delta.num_seconds() + 60 > expiration.expires_in
    }

    pub async fn update_token(&self, client_id: &str, scope: &str) -> Result<(), ServerError> {
        if !self.token_expires().await {
            return Ok(());
        }

        let mut expiration = self.expiration.write().await;

        let token = keymaster::get_token(&self.session, client_id, scope).await?;
        let mut rtoken = self
            .client
            .token
            .lock()
            .await
            .map_err(|e| ServerError::InnerError(format!("can't update token: {:?}", e)))?;
        *rtoken = Some(rspotify::Token {
            access_token: token.access_token.clone(),
            expires_in: chrono::Duration::seconds(token.expires_in.into()),
            scopes: HashSet::from_iter(token.scope.clone()),
            expires_at: None,
            refresh_token: None,
        });

        expiration.update_expires_in(token.expires_in as i64);
        Ok(())
    }

    /// AES-128 encryption with `SpotifyAccount.secret`
    pub fn encrypt(&self, buf: &[u8]) -> (Vec<u8>, [u8; 16]) {
        let iv: [u8; 16] = rand::random();
        let enc = crypto::encrypt_aes128(&self.secret, &iv, buf);
        (enc, iv)
    }

    /// AES-128 decryption with `SpotifyAccount.secret`
    pub fn decrypt(&self, iv: &[u8], buf: &[u8]) -> Result<Vec<u8>, ServerError> {
        crypto::decrypt_aes128(&self.secret, &iv, buf)
            .map_err(|e| ServerError::InnerError(format!("{:?}", e)))
    }
}

/// UserName Wrapper
#[derive(Hash, Eq, PartialEq, serde::Deserialize, Clone)]
pub struct UserName(String);

impl From<&str> for UserName {
    fn from(username: &str) -> Self {
        Self(username.to_string())
    }
}

impl From<String> for UserName {
    fn from(username: String) -> Self {
        Self(username)
    }
}

impl AsRef<str> for UserName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

/// Spotify Accounts at HashMap
#[derive(Default)]
pub struct SpotifyAccounts {
    inner: HashMap<UserName, SpotifyAccount>,
}

impl SpotifyAccounts {
    pub fn get(&self, username: impl Into<UserName>) -> Option<&SpotifyAccount> {
        self.inner.get(&username.into())
    }

    pub fn insert(&mut self, username: impl Into<UserName>, account: SpotifyAccount) {
        self.inner.insert(username.into(), account);
    }

    pub fn contains_key(&self, username: &UserName) -> bool {
        self.inner.contains_key(username)
    }

    pub fn keys(&self) -> impl IntoIterator<Item = &UserName> {
        self.inner.keys()
    }
}
