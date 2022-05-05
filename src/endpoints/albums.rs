use futures::StreamExt;

use actix_web::{web, HttpResponse};

use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{AlbumId, Id, Page, SavedAlbum, SimplifiedAlbum, SimplifiedTrack},
};

use crate::{
    account::SpotifyAccount,
    app_store::AppStore,
    endpoints::{
        params::{IdsData, LimitOffsetData},
        utils::{json_response, ok_response},
    },
    errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/albums/{id}`
pub async fn album(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let album_id = AlbumId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid album id: {}", id.as_str())))?;

    let result = account.client.album(&album_id).await?;
    json_response(&result)
}

/// Path: GET `/albums`
pub async fn albums(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account.client.albums(query.ids().iter()).await?;
    json_response(&result)
}

/// Path: GET `/albums/{id}/tracks`
pub async fn album_tracks(
    id: web::Path<String>,
    query: web::Query<LimitOffsetData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let album_id =
        AlbumId::from_id(id.as_str()).map_err(|_| ServerError::ParamsError(format!("{}", id)))?;

    if query.limit.is_some() {
        let page = page_tracks(&account, &album_id, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let tracks = all_tracks(&account, &album_id).await?;
        json_response(&tracks)
    }
}

/// Album all tracks
async fn all_tracks(
    account: &SpotifyAccount,
    album_id: &AlbumId,
) -> Result<Vec<SimplifiedTrack>, ServerError> {
    let mut track_stream = account.client.album_track(album_id);
    let mut tracks = vec![];
    while let Some(item) = track_stream.next().await {
        if item.is_err() {
            return Err(ServerError::RequestError(format!("{:?}", item)));
        } else {
            tracks.push(item.unwrap());
        }
    }
    Ok(tracks)
}

/// Album tracks by page
async fn page_tracks(
    account: &SpotifyAccount,
    album_id: &AlbumId,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<SimplifiedTrack>, ServerError> {
    let page = account
        .client
        .album_track_manual(album_id, limit, offset)
        .await?;
    Ok(page)
}

/// Path: GET `/me/albums`
pub async fn saved_albums(
    query: web::Query<LimitOffsetData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    if query.limit.is_some() {
        let page = page_saved_albums(&account, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let albums = all_saved_albums(&account).await?;
        json_response(&albums)
    }
}

/// Current user all saved albums
async fn all_saved_albums(account: &SpotifyAccount) -> Result<Vec<SavedAlbum>, ServerError> {
    let mut album_stream = account.client.current_user_saved_albums(None);
    let mut albums = vec![];
    while let Some(item) = album_stream.next().await {
        if item.is_err() {
            return Err(ServerError::RequestError(format!("{:?}", item)));
        } else {
            albums.push(item.unwrap());
        }
    }
    Ok(albums)
}

/// Current user saved albums by page
async fn page_saved_albums(
    account: &SpotifyAccount,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<SavedAlbum>, ServerError> {
    let page = account
        .client
        .current_user_saved_albums_manual(None, limit, offset)
        .await?;
    Ok(page)
}

/// Path: PUT `/me/albums`
pub async fn save_albums(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    account
        .client
        .current_user_saved_albums_add(query.ids().iter())
        .await?;
    ok_response()
}

/// Path: DELETE `/me/albums`
pub async fn delete_albums(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    account
        .client
        .current_user_saved_albums_delete(query.ids().iter())
        .await?;
    ok_response()
}

/// Path: GET `/browse/new-releases`
pub async fn new_releases(
    query: web::Query<LimitOffsetData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    if query.limit.is_some() {
        let albums = all_new_releases(&account).await?;
        json_response(&albums)
    } else {
        let page = page_new_releases(&account, query.limit, query.offset).await?;
        json_response(&page)
    }
}

/// All new releases albums
async fn all_new_releases(account: &SpotifyAccount) -> Result<Vec<SimplifiedAlbum>, ServerError> {
    let mut album_stream = account.client.new_releases(None);
    let mut albums = vec![];
    while let Some(item) = album_stream.next().await {
        if item.is_err() {
            return Err(ServerError::RequestError(format!("{:?}", item)));
        } else {
            albums.push(item.unwrap());
        }
    }
    Ok(albums)
}

/// New releases albums by page
async fn page_new_releases(
    account: &SpotifyAccount,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<SimplifiedAlbum>, ServerError> {
    let page = account
        .client
        .new_releases_manual(None, limit, offset)
        .await?;
    Ok(page)
}
