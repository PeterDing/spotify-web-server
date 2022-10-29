use actix_web::{web, HttpResponse};

use rspotify::{
    clients::{BaseClient, OAuthClient},
    http::Query,
};

use crate::{
    app_store::AppStore,
    endpoints::utils::{json_response, ok_with_body_response},
    errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/me`
/// Get detailed profile information about the current user
/// (including the current user's username).
#[tracing::instrument(skip(app_store, session))]
pub async fn me(
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account.client.me().await?;
    json_response(&result)
}

/// Path: GET `/users/{id}`
/// Get public profile information about a Spotify user.
#[tracing::instrument(skip(app_store, session))]
pub async fn user(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account
        .client
        .endpoint_get(&format!("users/{}", id), &Query::new())
        .await?;
    ok_with_body_response(result)
}
