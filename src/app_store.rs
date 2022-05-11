use std::path::PathBuf;

use librespot::{core::cache::Cache, discovery::Credentials};
use tokio::sync::{self, RwLockReadGuard};
use url::Url;

use crate::{
    account::{utils::load_credentials, SpotifyAccount, SpotifyAccounts, UserName},
    errors::ServerError,
};

pub const DEFAULT_CLIENT_ID: &str = "06e33fd028714708827e268040efb778";
pub const DEFAULT_SCOPE: &str = "user-read-private,playlist-read-private,playlist-read-collaborative,playlist-modify-public,playlist-modify-private,user-follow-modify,user-follow-read,user-library-read,user-library-modify,user-top-read,user-read-recently-played";

/// App Store Data
/// It stores `SpotifyAccounts` and a global `Mutex`
pub struct AppStore {
    pub spotify_accounts: sync::RwLock<SpotifyAccounts>,
    pub client_id: String,
    pub cache_dir: PathBuf,
    pub proxy: Option<Url>,
}

impl AppStore {
    pub fn new(client_id: &str, cache_dir: &str, proxy: Option<Url>) -> Self {
        Self {
            spotify_accounts: sync::RwLock::new(SpotifyAccounts::default()),
            client_id: client_id.to_string(),
            cache_dir: PathBuf::from(cache_dir),
            proxy,
        }
    }

    pub async fn load_cache(&self) -> Result<(), ServerError> {
        for entry in self.cache_dir.read_dir()?.flatten() {
            let creds_dir = entry.path();
            if let Some(credentials) = load_credentials(creds_dir.clone()) {
                let username = creds_dir.file_name().unwrap().to_str().unwrap();
                let cache = Cache::new(Some(creds_dir.clone()), None, None)?;
                let account = SpotifyAccount::new(credentials, cache, self.proxy.clone()).await?;
                self.insert_account(username, account).await;
            }
        }
        Ok(())
    }

    pub async fn create_account(
        &self,
        username: &str,
        password: &str,
        to_cache: bool,
    ) -> Result<(), ServerError> {
        let cred_dir = if to_cache {
            Some(self.cache_dir.join(username))
        } else {
            None
        };

        let credentials = if let Some(ref cd) = cred_dir {
            load_credentials(cd).unwrap_or_else(|| Credentials::with_password(username, password))
        } else {
            Credentials::with_password(username, password)
        };

        let account = SpotifyAccount::create(credentials, cred_dir, self.proxy.clone()).await?;
        self.insert_account(username, account).await;

        Ok(())
    }

    pub async fn insert_account(&self, username: impl Into<UserName>, account: SpotifyAccount) {
        let mut spotify_accounts = self.spotify_accounts.write().await;
        spotify_accounts.insert(username, account);
    }

    pub async fn authorize<'a>(
        &'a self,
        username: impl Into<UserName>,
    ) -> Result<RwLockReadGuard<'_, SpotifyAccount>, ServerError> {
        let spotify_accounts = self.spotify_accounts.read().await;

        let account = RwLockReadGuard::try_map(spotify_accounts, |sa| sa.get(username));
        match account {
            Ok(a) => {
                // Update Token when it expires
                a.update_token(&self.client_id, DEFAULT_SCOPE).await?;
                Ok(a)
            }
            Err(_) => Err(ServerError::AuthenticationError),
        }
    }
}
