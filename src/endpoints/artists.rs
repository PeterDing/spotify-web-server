use futures::StreamExt;

use actix_web::{web, HttpResponse};

use rspotify::{
    clients::BaseClient,
    model::{ArtistId, Page, SimplifiedAlbum},
};

use crate::{
    account::SpotifyAccount,
    app_store::AppStore,
    endpoints::{
        params::{IdsData, LimitOffsetData},
        utils::json_response,
    },
    errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/artists/{id}`
/// Get Spotify catalog information for a single artist identified by their unique Spotify ID.
#[tracing::instrument(skip(app_store, session))]
pub async fn artist(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let id_str = id.into_inner();

    let artist_id = ArtistId::from_id(id_str.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid artist id: {}", id_str)))?;

    let result = account.client.artist(artist_id).await?;
    json_response(&result)
}

/// Path: GET `/artists`
/// Get Spotify catalog information for several artists based on their Spotify IDs.
#[tracing::instrument(skip(app_store, session))]
pub async fn artists(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let artist_ids = crate::into_ids!(ArtistId, query.ids());
    let result = account.client.artists(artist_ids).await?;
    json_response(&result)
}

/// Path: GET `/artists/{id}/albums`
/// Get Spotify catalog information about an artist's albums.
#[tracing::instrument(skip(app_store, session))]
pub async fn artist_albums(
    id: web::Path<String>,
    query: web::Query<LimitOffsetData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let id_str = id.into_inner();

    let artist_id = ArtistId::from_id(id_str.as_str())
        .map_err(|_| ServerError::ParamsError(format!("{}", id_str)))?;

    if query.limit.is_some() {
        let page = page_albums(&account, artist_id, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let artists = all_albums(&account, artist_id).await?;
        json_response(&artists)
    }
}

/// Artist all albums
async fn all_albums(
    account: &SpotifyAccount,
    artist_id: ArtistId<'_>,
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
    artist_id: ArtistId<'_>,
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
/// Get Spotify catalog information about an artist's top tracks by country.
#[tracing::instrument(skip(app_store, session))]
pub async fn artist_top_tracks(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let id_str = id.into_inner();

    let artist_id = ArtistId::from_id(id_str.as_str())
        .map_err(|_| ServerError::ParamsError(format!("{}", id_str)))?;
    let tracks = account
        .client
        .artist_top_tracks(artist_id, Some(rspotify::model::Market::FromToken))
        .await?;
    json_response(&tracks)
}

/// Path: GET `/artists/{id}/related-artists`
/// Get Spotify catalog information about artists similar to a given artist.
/// Similarity is based on analysis of the Spotify community's listening history.
#[tracing::instrument(skip(app_store, session))]
pub async fn artist_related_artists(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let id_str = id.into_inner();

    let artist_id = ArtistId::from_id(id_str.as_str())
        .map_err(|_| ServerError::ParamsError(format!("{}", id_str)))?;
    let artists = account.client.artist_related_artists(artist_id).await?;
    json_response(&artists)
}
