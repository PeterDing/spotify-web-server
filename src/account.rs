use std::collections::{HashMap, HashSet};

use librespot::core::{
    authentication::Credentials, config::SessionConfig, keymaster, session::Session,
};

use rspotify::AuthCodeSpotify;

use url::Url;

use crate::errors::ServerError;

const CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";
const SCOPE: &str = "user-read-private,playlist-read-private,playlist-read-collaborative,playlist-modify-public,playlist-modify-private,user-follow-modify,user-follow-read,user-library-read,user-library-modify,user-top-read,user-read-recently-played";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Token {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,
    pub scope: Vec<String>,
}

/// From `Token` to `rspotify::Token`
impl From<Token> for rspotify::Token {
    fn from(token: Token) -> Self {
        rspotify::Token {
            access_token: token.access_token.clone(),
            expires_in: chrono::Duration::seconds(token.expires_in.into()),
            scopes: HashSet::from_iter(token.scope.clone()),
            expires_at: None,
            refresh_token: None,
        }
    }
}

/// From `keymaster::Token` to Token
impl From<keymaster::Token> for Token {
    fn from(ktoken: keymaster::Token) -> Self {
        let keymaster::Token {
            access_token,
            expires_in,
            token_type,
            scope,
        } = ktoken;
        Token {
            access_token,
            expires_in,
            token_type,
            scope,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub user_agent: String,
    pub device_id: String,
    pub proxy: Option<String>,
    pub ap_port: Option<u16>,
}

/// From `Config` to `SessionConfig`
impl From<Config> for SessionConfig {
    fn from(config: Config) -> Self {
        let proxy = config
            .proxy
            .map(|p| Url::parse(&p).expect("Proxy url is invalid"));
        let Config {
            user_agent,
            device_id,
            ap_port,
            ..
        } = config;
        SessionConfig {
            user_agent,
            device_id,
            proxy,
            ap_port,
        }
    }
}

/// From `SessionConfig` to `Config`
impl From<SessionConfig> for Config {
    fn from(sconfig: SessionConfig) -> Self {
        let SessionConfig {
            user_agent,
            device_id,
            proxy,
            ap_port,
        } = sconfig;
        Config {
            user_agent,
            device_id,
            proxy: proxy.map(|p| p.to_string()),
            ap_port,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Auth {
    pub token: Token,
    pub credentials: Credentials,
    pub config: Config,
}

pub struct SpotifyAccount {
    pub auth: Auth,
    pub session: Session,
    pub client: AuthCodeSpotify,
}

impl SpotifyAccount {
    pub async fn new(auth: Auth) -> Result<Self, ServerError> {
        let Auth {
            token,
            credentials,
            config,
        } = auth.clone();
        let session = Session::connect(config.into(), credentials, None).await?;
        let client: AuthCodeSpotify = AuthCodeSpotify::from_token(token.into());
        let account = SpotifyAccount {
            auth,
            session,
            client,
        };

        Ok(account)
    }

    pub async fn login(
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, ServerError> {
        let credentials = Credentials::with_password(username, password);
        let config = SessionConfig::default();
        let session = Session::connect(config.clone(), credentials.clone(), None).await?;
        let token = keymaster::get_token(&session, CLIENT_ID, SCOPE).await?;
        let auth = Auth {
            token: token.into(),
            credentials,
            config: config.into(),
        };
        SpotifyAccount::new(auth).await
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
