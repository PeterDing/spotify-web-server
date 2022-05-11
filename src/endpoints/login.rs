use actix_web::{web, HttpResponse};

use crate::{
    account::UserName,
    app_store::AppStore,
    endpoints::params::{LoginData, UserNameData},
    errors::ServerError,
    session::ServerSession,
};

#[tracing::instrument(skip(app_store, session))]
pub async fn login(
    form: web::Form<LoginData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let to_cache = form.cache.map(|v| if v == 1 { v } else { 0 }).unwrap_or(0) == 1;

    app_store
        .create_account(&form.username, &form.password, to_cache)
        .await?;

    session.insert_username(&form.username)?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(skip(app_store, session))]
pub async fn miracle(
    query: web::Query<UserNameData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    if let Some(username) = &query.username {
        let accounts = app_store.spotify_accounts.read().await;
        if accounts.contains_key(&UserName::from(username.as_str())) {
            session.insert_username(username)?;
            Ok(HttpResponse::Ok().finish())
        } else {
            Err(ServerError::NoLoginError)
        }
    } else {
        let accounts = app_store.spotify_accounts.read().await;
        let username = accounts.keys().into_iter().next();
        if let Some(one) = username {
            session.insert_username(one.as_ref())?;
            Ok(HttpResponse::Ok().finish())
        } else {
            Err(ServerError::NoLoginError)
        }
    }
}
