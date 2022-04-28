use futures::StreamExt;

use actix_web::{web, HttpResponse};

use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{Id, Page, SavedTrack, TrackId},
};

use crate::{
    account::SpotifyAccount,
    app_store::AppStore,
    errors::ServerError,
    routes::{
        params::{IdsQueryData, PageQueryData},
        utils::{json_response, ok_response},
    },
    session::ServerSession,
};

/// Path: GET `/tracks/{id}`
pub async fn track(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let track_id = TrackId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid track id: {}", id.as_str())))?;

    let result = account.client.track(&track_id).await?;
    json_response(&result)
}

/// Path: GET `/tracks`
pub async fn tracks(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account.client.tracks(query.ids().iter(), None).await?;
    json_response(&result)
}

/// Path: GET `/me/tracks`
pub async fn saved_tracks(
    query: web::Query<PageQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    if query.limit.is_some() {
        let page = page_saved_tracks(&account, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let tracks = all_saved_tracks(&account).await?;
        json_response(&tracks)
    }
}

/// Current user all saved tracks
async fn all_saved_tracks(account: &SpotifyAccount) -> Result<Vec<SavedTrack>, ServerError> {
    let mut track_stream = account.client.current_user_saved_tracks(None);
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

/// Current user saved tracks by page
async fn page_saved_tracks(
    account: &SpotifyAccount,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<SavedTrack>, ServerError> {
    let page = account
        .client
        .current_user_saved_tracks_manual(None, limit, offset)
        .await?;
    Ok(page)
}

/// Path: PUT `/me/tracks`
pub async fn save_tracks(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    account
        .client
        .current_user_saved_tracks_add(query.ids().iter())
        .await?;
    ok_response()
}

/// Path: DELETE `/me/tracks`
pub async fn delete_tracks(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    account
        .client
        .current_user_saved_tracks_delete(query.ids().iter())
        .await?;
    ok_response()
}

/// Path: GET `/audio-features/{id}`
pub async fn track_features(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let track_id = TrackId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid track id: {}", id.as_str())))?;

    let result = account.client.track_features(&track_id).await?;
    json_response(&result)
}

/// Path: GET `/audio-features`
pub async fn tracks_features(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account.client.tracks_features(query.ids().iter()).await?;
    json_response(&result)
}

/// Path: GET `/audio-analysis/{id}`
pub async fn track_analysis(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let track_id = TrackId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid track id: {}", id.as_str())))?;

    let result = account.client.track_analysis(&track_id).await?;
    json_response(&result)
}
