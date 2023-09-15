use actix_web::{web, HttpResponse};

use rspotify::{clients::BaseClient, http::Query};

use crate::{
    app_store::AppStore, endpoints::utils::ok_with_body_response, errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/markets`
/// Get the list of markets where Spotify is available.
#[tracing::instrument(skip(app_store, session))]
pub async fn markets(
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let url = "markets";
    let result = account.client.api_get(url, &Query::new()).await?;

    ok_with_body_response(result)
}
