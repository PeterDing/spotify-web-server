use tokio::sync::RwLockReadGuard;

use crate::{
    account::SpotifyAccount, app_store::AppStore, errors::ServerError, session::ServerSession,
};

pub async fn authorize<'a>(
    app_store: &'a AppStore,
    session: &'a ServerSession,
) -> Result<RwLockReadGuard<'a, SpotifyAccount>, ServerError> {
    let username = session.get_username()?;
    app_store.authorize(username).await
}
