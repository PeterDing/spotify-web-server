use actix_web::{web, HttpResponse};

use rspotify::{
    clients::BaseClient,
    model::{ArtistId, TrackId},
};

use crate::{
    app_store::AppStore,
    endpoints::{params::RecommendationsData, utils::json_response},
    errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/recommendations`
pub async fn recommendations(
    query: web::Query<RecommendationsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account
        .client
        .recommendations(
            query.attributes(),
            query.seed_artists::<ArtistId>().as_ref(),
            query.seed_genres(),
            query.seed_tracks::<TrackId>().as_ref(),
            None,
            query.limit(),
        )
        .await?;
    json_response(&result)
}
