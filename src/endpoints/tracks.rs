use futures::StreamExt;

use actix_web::{web, HttpResponse};

use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{Page, SavedTrack, TrackId},
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

/// Path: GET `/tracks/{id}`
/// Get Spotify catalog information for a single track identified by its unique Spotify ID.
#[tracing::instrument(skip(app_store, session))]
pub async fn track(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let id_str = id.into_inner();

    let track_id = TrackId::from_id(id_str.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid track id: {}", id_str)))?;

    let result = account.client.track(track_id, None).await?;
    json_response(&result)
}

/// Path: GET `/tracks`
/// Get Spotify catalog information for multiple tracks based on their Spotify IDs.
#[tracing::instrument(skip(app_store, session))]
pub async fn tracks(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let track_ids = crate::into_ids!(TrackId, query.ids());
    let result = account.client.tracks(track_ids, None).await?;
    json_response(&result)
}

/// Path: GET `/me/tracks`
/// Get a list of the songs saved in the current Spotify user's 'Your Music' library.
#[tracing::instrument(skip(app_store, session))]
pub async fn saved_tracks(
    query: web::Query<LimitOffsetData>,
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
/// Save one or more tracks to the current user's 'Your Music' library.
#[tracing::instrument(skip(app_store, session))]
pub async fn save_tracks(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let track_ids = crate::into_ids!(TrackId, query.ids());
    account
        .client
        .current_user_saved_tracks_add(track_ids)
        .await?;
    ok_response()
}

/// Path: DELETE `/me/tracks`
/// Remove one or more tracks from the current user's 'Your Music' library.
#[tracing::instrument(skip(app_store, session))]
pub async fn delete_tracks(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let track_ids = crate::into_ids!(TrackId, query.ids());
    account
        .client
        .current_user_saved_tracks_delete(track_ids)
        .await?;
    ok_response()
}

/// Path: GET `/audio-features/{id}`
/// Get audio feature information for a single track identified by its unique Spotify ID.
#[tracing::instrument(skip(app_store, session))]
pub async fn track_features(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let id_str = id.into_inner();

    let track_id = TrackId::from_id(id_str.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid track id: {}", id_str)))?;

    let result = account.client.track_features(track_id).await?;
    json_response(&result)
}

/// Path: GET `/audio-features`
/// Get audio features for multiple tracks based on their Spotify IDs.
#[tracing::instrument(skip(app_store, session))]
pub async fn tracks_features(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let track_ids = crate::into_ids!(TrackId, query.ids());
    let result = account.client.tracks_features(track_ids).await?;
    json_response(&result)
}

/// Path: GET `/audio-analysis/{id}`
/// Get a low-level audio analysis for a track in the Spotify catalog.
/// The audio analysis describes the track’s structure and musical
/// content, including rhythm, pitch, and timbre.
#[tracing::instrument(skip(app_store, session))]
pub async fn track_analysis(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;
    let id_str = id.into_inner();

    let track_id = TrackId::from_id(id_str.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid track id: {}", id_str)))?;

    let result = account.client.track_analysis(track_id).await?;
    json_response(&result)
}
