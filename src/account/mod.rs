use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::Duration,
};

use tokio::{sync, time::timeout};

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
    pub session: RwLock<Session>,
    pub client: AuthCodeSpotify,
    expiration: RwLock<Expiration>,
    cache: Option<Cache>,
    proxy: Option<Url>,
    // Secret key
    secret: [u8; 16],
    lock: sync::Mutex<()>,
}

impl SpotifyAccount {
    pub async fn new(
        credentials: Credentials,
        cache: Cache,
        proxy: Option<Url>,
    ) -> Result<Self, ServerError> {
        let config = SessionConfig {
            proxy: proxy.clone(),
            ..Default::default()
        };
        let (session, credentials) =
            Session::connect(config, credentials.clone(), Some(cache.clone()), true).await?;

        let client: AuthCodeSpotify = AuthCodeSpotify::default();
        let secret: [u8; 16] = rand::random();
        let account = SpotifyAccount {
            credentials,
            session: RwLock::new(session),
            client,
            expiration: RwLock::new(Expiration::default()),
            cache: Some(cache),
            proxy,
            secret,
            lock: sync::Mutex::new(()),
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
        let cache = Cache::new(cache_dir, None, None, None)?;
        SpotifyAccount::new(credentials, cache, proxy).await
    }

    async fn token_expires(&self) -> bool {
        let expiration = self.expiration.read().await;
        let delta = Utc::now() - expiration.token_expiration;
        delta.num_seconds() + 10 * 60 > expiration.expires_in
    }

    async fn set_token(&self, token: keymaster::Token) -> Result<(), ServerError> {
        println!(
            "----------------\n================ toke.expires_in: {}\n----------------",
            token.expires_in
        );

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

        let mut expiration = self.expiration.write().await;
        expiration.update_expires_in(token.expires_in as i64);
        Ok(())
    }

    pub async fn reset_session(&self, client_id: &str, scope: &str) {
        loop {
            tracing::info!("Reset librespot session");
            let config = SessionConfig {
                proxy: self.proxy.clone(),
                ..Default::default()
            };

            let fut = Session::connect(config, self.credentials.clone(), self.cache.clone(), true);
            if let Ok(Ok((new_session, _))) = timeout(Duration::from_secs(5), fut).await {
                let mut session = self.session.write().await;
                session.shutdown();
                *session = new_session;
                if let Ok(token) = keymaster::get_token(&session, client_id, scope).await {
                    self.set_token(token).await.unwrap();
                }
                break;
            } else {
                tracing::warn!("Fail reset session");
            }
        }
    }

    pub async fn update_token(&self, client_id: &str, scope: &str) -> Result<(), ServerError> {
        let lock = self.lock.lock().await;

        if !self.token_expires().await {
            return Ok(());
        }

        tracing::info!("Token expires");
        let session = self.session.read().await;
        if let Ok(token) = keymaster::get_token(&session, client_id, scope).await {
            return self.set_token(token).await;
        }

        tracing::warn!("keymaster::get_token fails");

        drop(session);

        // This is MercuryError. There is no idea why it occurs.
        // So, we just force to reset the session.
        self.reset_session(client_id, scope).await;

        Ok(())
    }

    pub async fn retry_update_token(
        &self,
        client_id: &str,
        scope: &str,
        retries: usize,
    ) -> Result<(), ServerError> {
        for i in 0..retries {
            if i > 0 {
                tracing::warn!("Retry update token by {}", i);
            }
            match timeout(Duration::from_secs(10), self.update_token(client_id, scope)).await {
                Ok(result) => return result,
                Err(err) => {
                    if i + 1 == retries {
                        return Err(ServerError::InnerError(format!(
                            "Failed to update token after {} retries, the last error is {}",
                            retries, err
                        )));
                    }
                }
            }
        }
        return Err(ServerError::InnerError(format!(
            "Failed to update token after {} retries",
            retries
        )));
    }

    /// AES-128 encryption with `SpotifyAccount.secret`
    pub fn encrypt(&self, buf: &[u8]) -> (Vec<u8>, [u8; 16]) {
        let iv: [u8; 16] = rand::random();
        let enc = crypto::encrypt_aes128(&self.secret, &iv, buf);
        (enc, iv)
    }

    /// AES-128 decryption with `SpotifyAccount.secret`
    pub fn decrypt(&self, iv: &[u8], buf: &[u8]) -> Result<Vec<u8>, ServerError> {
        crypto::decrypt_aes128(&self.secret, iv, buf)
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
