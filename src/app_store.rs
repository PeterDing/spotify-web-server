use tokio::sync::{self, RwLockReadGuard};

use crate::{
    account::{SpotifyAccount, SpotifyAccounts, UserName},
    errors::ServerError,
};

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
            Ok(a) => {
                a.update_token().await?;
                Ok(a)
            }
            Err(_) => Err(ServerError::AuthenticationError),
        }
    }

    pub async fn insert_account(&self, username: impl Into<UserName>, account: SpotifyAccount) {
        let mut spotify_accounts = self.spotify_accounts.write().await;
        spotify_accounts.insert_account(username, account);
    }
}
