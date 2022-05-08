use actix_web::{web, HttpResponse};

use rspotify::clients::BaseClient;

use crate::{
    app_store::AppStore,
    endpoints::{
        params::{LimitOffsetData, SearchData},
        utils::json_response,
    },
    errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/search`
/// Get Spotify catalog information about albums, artists, playlists,
/// tracks, shows or episodes that match a keyword string.
pub async fn search(
    query: web::Query<SearchData>,
    limit_offset: web::Query<LimitOffsetData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account
        .client
        .search(
            &query.q,
            &query.type_,
            query.market.as_ref(),
            query.include_external.as_ref(),
            limit_offset.limit,
            limit_offset.offset,
        )
        .await?;

    json_response(result)
}
