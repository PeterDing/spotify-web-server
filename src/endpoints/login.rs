use std::path::Path;

use actix_web::{web, HttpResponse};

use librespot::core::authentication::Credentials;

use crate::{
    account::{
        utils::{load_credentials, CONFIG_ROOT},
        SpotifyAccount, UserName,
    },
    app_store::AppStore,
    endpoints::params::{LoginFormData, UserNameQueryData},
    errors::ServerError,
    session::ServerSession,
};

pub async fn login(
    form: web::Form<LoginFormData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let to_cache = form.cache.map(|v| if v == 1 { v } else { 0 }).unwrap_or(0);

    let cache_dir = match to_cache {
        1 => Some(Path::new(CONFIG_ROOT).join(&form.username)),
        _ => None,
    };

    let credentials = if let Some(ref cd) = cache_dir {
        load_credentials(cd).unwrap_or_else(|| {
            Credentials::with_password(form.username.clone(), form.password.clone())
        })
    } else {
        Credentials::with_password(form.username.clone(), form.password.clone())
    };

    let account = SpotifyAccount::create(credentials, cache_dir).await?;
    app_store
        .insert_account(form.username.clone(), account)
        .await;
    session.insert_username(&form.username)?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn miracle(
    query: web::Query<UserNameQueryData>,
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
