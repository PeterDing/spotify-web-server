use futures::StreamExt;

use actix_web::{web, HttpResponse};

use rspotify::{
    clients::BaseClient,
    model::{ArtistId, Id, Page, SimplifiedAlbum},
};

use crate::{
    account::SpotifyAccount,
    app_store::AppStore,
    endpoints::{
        params::{IdsQueryData, PageQueryData},
        utils::json_response,
    },
    errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/artists/{id}`
pub async fn artist(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let artist_id = ArtistId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid artist id: {}", id.as_str())))?;

    let result = account.client.artist(&artist_id).await?;
    json_response(&result)
}

/// Path: GET `/artists`
pub async fn artists(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account.client.artists(query.ids().iter()).await?;
    json_response(&result)
}

/// Path: GET `/artists/{id}/albums`
pub async fn artist_albums(
    id: web::Path<String>,
    query: web::Query<PageQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let artist_id =
        ArtistId::from_id(id.as_str()).map_err(|_| ServerError::ParamsError(format!("{}", id)))?;

    if query.limit.is_some() {
        let page = page_albums(&account, &artist_id, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let artists = all_albums(&account, &artist_id).await?;
        json_response(&artists)
    }
}

/// Artist all albums
async fn all_albums(
    account: &SpotifyAccount,
    artist_id: &ArtistId,
) -> Result<Vec<SimplifiedAlbum>, ServerError> {
    let mut album_stream = account.client.artist_albums(artist_id, None, None);
    let mut artists = vec![];
    while let Some(item) = album_stream.next().await {
        if item.is_err() {
            return Err(ServerError::RequestError(format!("{:?}", item)));
        } else {
            artists.push(item.unwrap());
        }
    }
    Ok(artists)
}

/// Artist albums by page
async fn page_albums(
    account: &SpotifyAccount,
    artist_id: &ArtistId,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<SimplifiedAlbum>, ServerError> {
    let page = account
        .client
        .artist_albums_manual(artist_id, None, None, limit, offset)
        .await?;
    Ok(page)
}

/// Path: GET `/artists/{id}/top-tracks`
pub async fn artist_top_tracks(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let artist_id =
        ArtistId::from_id(id.as_str()).map_err(|_| ServerError::ParamsError(format!("{}", id)))?;
    let tracks = account
        .client
        .artist_top_tracks(&artist_id, &rspotify::model::Market::FromToken)
        .await?;
    json_response(&tracks)
}

/// Path: GET `/artists/{id}/related-artists`
pub async fn artist_related_artists(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let artist_id =
        ArtistId::from_id(id.as_str()).map_err(|_| ServerError::ParamsError(format!("{}", id)))?;
    let artists = account.client.artist_related_artists(&artist_id).await?;
    json_response(&artists)
}
