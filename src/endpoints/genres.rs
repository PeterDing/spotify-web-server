use actix_web::{web, HttpResponse};

use rspotify::{clients::BaseClient, http::Query};

use crate::{
    app_store::AppStore, endpoints::utils::ok_with_body_response, errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/recommendations/available-genre-seeds`
/// Retrieve a list of available genres seed parameter values for recommendations.
#[tracing::instrument(skip(app_store, session))]
pub async fn genres(
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let url = "recommendations/available-genre-seeds";
    let result = account.client.endpoint_get(url, &Query::new()).await?;

    ok_with_body_response(result)
}
