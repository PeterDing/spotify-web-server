use futures::StreamExt;

use chrono::{TimeZone, Utc};

use actix_web::{web, HttpResponse};

use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{
        EpisodeId, Id, Page, PlayableId, PlaylistId, PlaylistItem, SimplifiedPlaylist, TrackId,
        UserId,
    },
};

use crate::{
    account::SpotifyAccount,
    app_store::AppStore,
    endpoints::{
        params::PageQueryData,
        utils::{json_response, ok_response},
    },
    errors::ServerError,
    session::ServerSession,
};

#[derive(serde::Deserialize)]
pub struct FieldsQueryData {
    fields: Option<String>,
}

/// Path: GET `/playlists/{id}`
/// Get a playlist owned by a Spotify user.
pub async fn playlist(
    id: web::Path<String>,
    fields_query: web::Query<FieldsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let playlist_id = PlaylistId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid playlist id: {}", id.as_str())))?;
    let fields = fields_query.fields.as_deref();

    let result = account.client.playlist(&playlist_id, fields, None).await?;
    json_response(&result)
}

#[derive(serde::Deserialize)]
pub struct PlaylistDescJsonData {
    name: Option<String>,
    public: Option<bool>,
    collaborative: Option<bool>,
    description: Option<String>,
}

/// Path: PUT `/playlists/{id}`
/// Change a playlist's name and public/private state.
/// (The user must, of course, own the playlist.)
pub async fn change_playlist_detail(
    id: web::Path<String>,
    json: web::Json<PlaylistDescJsonData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let playlist_id = PlaylistId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid playlist id: {}", id.as_str())))?;

    let result = account
        .client
        .playlist_change_detail(
            &playlist_id,
            json.name.as_deref(),
            json.public,
            json.description.as_deref(),
            json.collaborative,
        )
        .await?;
    json_response(&result)
}

/// Path: GET `/playlists/{id}/tracks`
/// Get full details of the items of a playlist owned by a Spotify user.
pub async fn playlist_tracks(
    id: web::Path<String>,
    query: web::Query<PageQueryData>,
    fields_query: web::Query<FieldsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let playlist_id = PlaylistId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("{}", id)))?;
    let fields = fields_query.fields.as_deref();

    if query.limit.is_some() {
        let page = page_tracks(&account, &playlist_id, fields, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let tracks = all_tracks(&account, &playlist_id, fields).await?;
        json_response(&tracks)
    }
}

/// Playlist all tracks
async fn all_tracks(
    account: &SpotifyAccount,
    playlist_id: &PlaylistId,
    fields: Option<&str>,
) -> Result<Vec<PlaylistItem>, ServerError> {
    let mut track_stream = account.client.playlist_items(playlist_id, fields, None);
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

/// Playlist tracks by page
async fn page_tracks(
    account: &SpotifyAccount,
    playlist_id: &PlaylistId,
    fields: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<PlaylistItem>, ServerError> {
    let page = account
        .client
        .playlist_items_manual(playlist_id, fields, None, limit, offset)
        .await?;
    Ok(page)
}

#[derive(serde::Deserialize)]
pub struct PlaylistAddItemQueryData {
    uris: String,
    position: Option<i32>,
}

impl PlaylistAddItemQueryData {
    fn items(&self) -> Vec<Box<dyn PlayableId>> {
        self.uris
            .split(',')
            .filter(|id_or_uri| {
                id_or_uri.starts_with("spotify:track:") || id_or_uri.starts_with("spotify:episode:")
            })
            .map(|id_or_uri| {
                if id_or_uri.starts_with("spotify:track:") {
                    TrackId::from_id_or_uri(id_or_uri).map(|id| Box::new(id) as Box<dyn PlayableId>)
                } else {
                    EpisodeId::from_id_or_uri(id_or_uri)
                        .map(|id| Box::new(id) as Box<dyn PlayableId>)
                }
            })
            .filter(|id| id.is_ok())
            .map(|id| id.unwrap())
            .collect()
    }
}

#[derive(serde::Deserialize)]
pub struct PlaylistAddItemJsonData {
    uris: Vec<String>,
    position: Option<i32>,
}

impl PlaylistAddItemJsonData {
    fn items(&self) -> Vec<Box<dyn PlayableId>> {
        self.uris
            .iter()
            .filter(|id_or_uri| {
                id_or_uri.starts_with("spotify:track:") || id_or_uri.starts_with("spotify:episode:")
            })
            .map(|id_or_uri| {
                if id_or_uri.starts_with("spotify:track:") {
                    TrackId::from_id_or_uri(id_or_uri).map(|id| Box::new(id) as Box<dyn PlayableId>)
                } else {
                    EpisodeId::from_id_or_uri(id_or_uri)
                        .map(|id| Box::new(id) as Box<dyn PlayableId>)
                }
            })
            .filter(|id| id.is_ok())
            .map(|id| id.unwrap())
            .collect()
    }
}

/// Path: POST `/playlists/{id}/tracks`
/// Add one or more items to a user's playlist.
pub async fn playlist_add_items(
    id: web::Path<String>,
    query: web::Query<PlaylistAddItemQueryData>,
    json: Option<web::Json<PlaylistAddItemJsonData>>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let playlist_id = PlaylistId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("{}", id)))?;

    if !query.uris.is_empty() {
        let items = query.items();
        let result = account
            .client
            .playlist_add_items(
                &playlist_id,
                items.iter().map(|item| item.as_ref()),
                query.position,
            )
            .await?;
        return json_response(&result);
    }
    if let Some(json) = json {
        let items = json.items();
        let result = account
            .client
            .playlist_add_items(
                &playlist_id,
                items.iter().map(|item| item.as_ref()),
                json.position,
            )
            .await?;
        return json_response(&result);
    }
    Err(ServerError::ParamsError(format!("No uris")))
}

/// Path: GET `/me/playlists`
/// Get a list of (or all) playlists owned or followed by the current Spotify user.
pub async fn current_user_playlists(
    query: web::Query<PageQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    if query.limit.is_some() {
        let page = page_current_user_playlists(&account, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let playlists = all_current_user_playlists(&account).await?;
        json_response(&playlists)
    }
}

/// Current user all saved playlists
async fn all_current_user_playlists(
    account: &SpotifyAccount,
) -> Result<Vec<SimplifiedPlaylist>, ServerError> {
    let mut playlist_stream = account.client.current_user_playlists();
    let mut playlists = vec![];
    while let Some(item) = playlist_stream.next().await {
        if item.is_err() {
            return Err(ServerError::RequestError(format!("{:?}", item)));
        } else {
            playlists.push(item.unwrap());
        }
    }
    Ok(playlists)
}

/// Current user saved playlists by page
async fn page_current_user_playlists(
    account: &SpotifyAccount,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<SimplifiedPlaylist>, ServerError> {
    let page = account
        .client
        .current_user_playlists_manual(limit, offset)
        .await?;
    Ok(page)
}

/// Path: GET `/users/{id}/playlists`
/// Get a list of the playlists owned or followed by a Spotify user.
pub async fn user_playlists(
    id: web::Path<String>,
    query: web::Query<PageQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let user_id =
        UserId::from_id(id.as_str()).map_err(|_| ServerError::ParamsError(format!("{}", id)))?;

    if query.limit.is_some() {
        let page = page_user_playlists(&account, &user_id, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let playlists = all_user_playlists(&account, &user_id).await?;
        json_response(&playlists)
    }
}

/// Current user all saved playlists
async fn all_user_playlists(
    account: &SpotifyAccount,
    user_id: &UserId,
) -> Result<Vec<SimplifiedPlaylist>, ServerError> {
    let mut playlist_stream = account.client.user_playlists(user_id);
    let mut playlists = vec![];
    while let Some(item) = playlist_stream.next().await {
        if item.is_err() {
            return Err(ServerError::RequestError(format!("{:?}", item)));
        } else {
            playlists.push(item.unwrap());
        }
    }
    Ok(playlists)
}

/// Current user saved playlists by page
async fn page_user_playlists(
    account: &SpotifyAccount,
    user_id: &UserId,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<SimplifiedPlaylist>, ServerError> {
    let page = account
        .client
        .user_playlists_manual(&user_id, limit, offset)
        .await?;
    Ok(page)
}

#[derive(serde::Deserialize)]
pub struct PublicJsonData {
    public: bool,
}

/// Path: PUT `/playlists/{id}/followers`
/// Add the current user as a follower of a playlist.
pub async fn follow_playlist(
    id: web::Path<String>,
    body: web::Bytes,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let playlist_id = PlaylistId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid playlist id: {}", id.as_str())))?;
    let public = if let Ok(p) = serde_json::from_slice::<PublicJsonData>(&body[..]) {
        Some(p.public)
    } else {
        Some(true)
    };

    account.client.playlist_follow(&playlist_id, public).await?;
    ok_response()
}

/// Path: DELETE `/playlists/{id}/followers`
/// Remove the current user as a follower of a playlist.
pub async fn unfollow_playlist(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let playlist_id = PlaylistId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid playlist id: {}", id.as_str())))?;

    account.client.playlist_unfollow(&playlist_id).await?;
    ok_response()
}

/// Path: POST `/users/{id}/playlists`
/// Create a playlist for a Spotify user.
/// (The playlist will be empty until you add tracks.)
pub async fn create_playlist(
    id: web::Path<String>,
    json: web::Json<PlaylistDescJsonData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let user_id = UserId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid user id: {}", id.as_str())))?;

    let name = if let Some(name) = &json.name {
        if name.is_empty() {
            return Err(ServerError::ParamsError("Missing playlist name".to_owned()));
        }
        name
    } else {
        return Err(ServerError::ParamsError("Missing playlist name".to_owned()));
    };

    let result = account
        .client
        .user_playlist_create(
            &user_id,
            name,
            json.public,
            json.collaborative,
            json.description.as_deref(),
        )
        .await?;
    json_response(&result)
}

#[derive(serde::Deserialize)]
pub struct FeatureQueryData {
    locale: Option<String>,
    country: Option<String>,
    timestamp: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

/// Path: GET `/browse/featured-playlists`
/// Get a list of the playlists owned or followed by a Spotify user.
pub async fn featured_playlists(
    query: web::Query<FeatureQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let timestamp = if let Some(ts) = &query.timestamp {
        if let Ok(ts) = Utc.datetime_from_str(ts, "%Y-%m-%dT%H:%M:%S") {
            Some(ts)
        } else {
            return Err(ServerError::ParamsError("Invalid timestamp".to_owned()));
        }
    } else {
        None
    };

    let result = account
        .client
        .featured_playlists(
            query.locale.as_deref(),
            None,
            timestamp.as_ref(),
            query.limit,
            query.offset,
        )
        .await?;
    json_response(&result)
}

/// Path: GET `/browse/categories/{id}/playlists`
/// Get a list of Spotify playlists tagged with a particular category.
pub async fn category_playlists(
    id: web::Path<String>,
    query: web::Query<PageQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    if query.limit.is_some() {
        let page = page_category_playlists(&account, &id, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let tracks = all_category_playlists(&account, &id).await?;
        json_response(&tracks)
    }
}

/// Category all playlists
async fn all_category_playlists(
    account: &SpotifyAccount,
    category_id: &str,
) -> Result<Vec<SimplifiedPlaylist>, ServerError> {
    let mut track_stream = account.client.category_playlists(category_id, None);
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

/// Category playlists by page
async fn page_category_playlists(
    account: &SpotifyAccount,
    category_id: &str,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<SimplifiedPlaylist>, ServerError> {
    let page = account
        .client
        .category_playlists_manual(category_id, None, limit, offset)
        .await?;
    Ok(page)
}
