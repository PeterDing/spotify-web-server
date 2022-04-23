use std::collections::{HashMap, HashSet};

use librespot::core::{
    authentication::Credentials, config::SessionConfig, keymaster, session::Session,
};

use rspotify::{clients::OAuthClient, model::TrackId, AuthCodeSpotify, Token};

use tokio::sync::{self, RwLockReadGuard};

use crate::errors::ServerError;

const CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";
const SCOPE: &str = "user-read-private,playlist-read-private,playlist-read-collaborative,playlist-modify-public,playlist-modify-private,user-follow-modify,user-follow-read,user-library-read,user-library-modify,user-top-read,user-read-recently-played";

pub struct SpotifyAccount {
    pub token: keymaster::Token,
    pub credentials: Credentials,
    pub session: Session,
    pub client: AuthCodeSpotify,
}

impl SpotifyAccount {
    pub async fn login(
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, ServerError> {
        let credentials = Credentials::with_password(username, password);
        let config = SessionConfig::default();
        let session = Session::connect(config, credentials.clone(), None).await?;
        let token = keymaster::get_token(&session, CLIENT_ID, SCOPE).await?;
        let client: AuthCodeSpotify = AuthCodeSpotify::from_token(Token {
            access_token: token.access_token.clone(),
            expires_in: chrono::Duration::seconds(token.expires_in.into()),
            scopes: HashSet::from_iter(token.scope.clone()),
            expires_at: None,
            refresh_token: None,
        });

        println!("token: {:?}", token);

        let account = SpotifyAccount {
            token,
            credentials,
            session,
            client,
        };

        Ok(account)
    }
}

/// UserName Wrapper
#[derive(Hash, Eq, PartialEq, serde::Deserialize)]
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

/// Spotify Accounts as HashMap
#[derive(Default)]
pub struct SpotifyAccounts {
    inner: HashMap<UserName, SpotifyAccount>,
}

impl SpotifyAccounts {
    pub fn get_account(&self, username: impl Into<UserName>) -> Option<&SpotifyAccount> {
        self.inner.get(&username.into())
    }

    pub fn insert_account(&mut self, username: impl Into<UserName>, account: SpotifyAccount) {
        self.inner.insert(username.into(), account);
    }
}

/// App Store Data
/// It stores `SpotifyAccounts` and a global `Mutex`
#[derive(Default)]
pub struct AppStore {
    pub spotify_accounts: sync::RwLock<SpotifyAccounts>,
}

impl AppStore {
    pub async fn authorize<'a>(
        &'a self,
        username: impl Into<UserName>,
    ) -> Result<RwLockReadGuard<'_, SpotifyAccount>, ServerError> {
        let spotify_accounts = self.spotify_accounts.read().await;

        let account = RwLockReadGuard::try_map(spotify_accounts, |sa| sa.get_account(username));
        match account {
            Ok(a) => Ok(a),
            Err(_) => Err(ServerError::AuthenticationError),
        }
    }

    pub async fn insert_account(&self, username: impl Into<UserName>, account: SpotifyAccount) {
        let mut spotify_accounts = self.spotify_accounts.write().await;
        spotify_accounts.insert_account(username, account);
    }
}
