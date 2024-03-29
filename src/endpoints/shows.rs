use futures::StreamExt;

use actix_web::{web, HttpResponse};

use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{Page, Show, ShowId, SimplifiedEpisode},
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

/// Path: GET `/shows/{id}`
#[tracing::instrument(skip(app_store, session))]
pub async fn show(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let show_id = ShowId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid show id: {}", id.as_str())))?;

    let result = account.client.get_a_show(show_id, None).await?;
    json_response(&result)
}

/// Path: GET `/shows`
#[tracing::instrument(skip(app_store, session))]
pub async fn shows(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let show_ids = crate::into_ids!(ShowId, query.ids());
    let result = account.client.get_several_shows(show_ids, None).await?;
    json_response(&result)
}

/// Path: GET `/shows/{id}/episodes`
#[tracing::instrument(skip(app_store, session))]
pub async fn show_episodes(
    id: web::Path<String>,
    query: web::Query<LimitOffsetData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let show_id =
        ShowId::from_id(id.as_str()).map_err(|_| ServerError::ParamsError(format!("{}", id)))?;

    if query.limit.is_some() {
        let page = page_episodes(&account, show_id, query.limit, query.offset).await?;
        json_response(&page)
    } else {
        let episodes = all_episodes(&account, show_id).await?;
        json_response(&episodes)
    }
}

/// Show all episodes
async fn all_episodes(
    account: &SpotifyAccount,
    show_id: ShowId<'_>,
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
    show_id: ShowId<'_>,
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
#[tracing::instrument(skip(app_store, session))]
pub async fn saved_shows(
    query: web::Query<LimitOffsetData>,
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
#[tracing::instrument(skip(app_store, session))]
pub async fn save_shows(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let show_ids = crate::into_ids!(ShowId, query.ids());
    account.client.save_shows(show_ids).await?;
    ok_response()
}

/// Path: DELETE `/me/shows`
#[tracing::instrument(skip(app_store, session))]
pub async fn delete_shows(
    query: web::Query<IdsData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let show_ids = crate::into_ids!(ShowId, query.ids());
    account
        .client
        .remove_users_saved_shows(show_ids, None)
        .await?;
    ok_response()
}
