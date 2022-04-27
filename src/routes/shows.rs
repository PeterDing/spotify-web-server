use futures::StreamExt;

use actix_web::{web, HttpResponse};

use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{Id, Page, Show, ShowId, SimplifiedEpisode},
};

use crate::{
    account::SpotifyAccount,
    app_store::AppStore,
    errors::ServerError,
    routes::utils::{json_response, ok_response},
    session::ServerSession,
};

/// Show Ids
#[derive(serde::Deserialize)]
pub struct IdsQueryData {
    ids: String,
}

impl IdsQueryData {
    fn show_ids<T: Id>(&self) -> Vec<T> {
        self.ids
            .split(',')
            .map(T::from_id)
            .filter(|id| id.is_ok())
            .map(|id| id.unwrap())
            .collect()
    }
}

/// Path: GET `/shows`
pub async fn shows(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let show_ids = query.show_ids();

    if show_ids.len() == 1 {
        let result = account.client.get_a_show(&show_ids[0], None).await?;
        json_response(&result)
    } else {
        let result = account
            .client
            .get_several_shows(show_ids.iter(), None)
            .await?;
        json_response(&result)
    }
}

/// Page Query Data
#[derive(serde::Deserialize)]
pub struct PageQueryData {
    limit: Option<u32>,
    offset: Option<u32>,
}

/// Path: GET `/shows/{id}/episodes`
pub async fn show_episodes(
    id: web::Path<String>,
    query: web::Query<PageQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let show_id =
        ShowId::from_id(id.as_str()).map_err(|_| ServerError::ParamsError(format!("{}", id)))?;

    if query.limit.is_some() {
        let page = page_episodes(&account, &show_id, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let episodes = all_episodes(&account, &show_id).await?;
        json_response(&episodes)
    }
}

/// Show all episodes
async fn all_episodes(
    account: &SpotifyAccount,
    show_id: &ShowId,
) -> Result<Vec<SimplifiedEpisode>, ServerError> {
    let mut episode_stream = account.client.get_shows_episodes(show_id, None);
    let mut episodes = vec![];
    while let Some(item) = episode_stream.next().await {
        if item.is_err() {
            return Err(ServerError::RequestError(format!("{:?}", item)));
        } else {
            episodes.push(item.unwrap());
        }
    }
    Ok(episodes)
}

/// Show episodes by page
async fn page_episodes(
    account: &SpotifyAccount,
    show_id: &ShowId,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<SimplifiedEpisode>, ServerError> {
    let page = account
        .client
        .get_shows_episodes_manual(show_id, None, limit, offset)
        .await?;
    Ok(page)
}

/// Path: GET `/me/shows`
pub async fn saved_shows(
    query: web::Query<PageQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    if query.limit.is_some() {
        let page = page_saved_shows(&account, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let shows = all_saved_shows(&account).await?;
        json_response(&shows)
    }
}

/// Current user all saved shows
async fn all_saved_shows(account: &SpotifyAccount) -> Result<Vec<Show>, ServerError> {
    let mut show_stream = account.client.get_saved_show();
    let mut shows = vec![];
    while let Some(item) = show_stream.next().await {
        if item.is_err() {
            return Err(ServerError::RequestError(format!("{:?}", item)));
        } else {
            shows.push(item.unwrap());
        }
    }
    Ok(shows)
}

/// Current user saved shows by page
async fn page_saved_shows(
    account: &SpotifyAccount,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<Show>, ServerError> {
    let page = account.client.get_saved_show_manual(limit, offset).await?;
    Ok(page)
}

/// Path: PUT `/me/shows`
pub async fn save_shows(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    account.client.save_shows(query.show_ids().iter()).await?;
    ok_response()
}

/// Path: DELETE `/me/shows`
pub async fn delete_shows(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    account
        .client
        .remove_users_saved_shows(query.show_ids().iter(), None)
        .await?;
    ok_response()
}
