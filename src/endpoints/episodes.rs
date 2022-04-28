use std::collections::HashMap;

use actix_web::{http::header::ContentType, web, HttpResponse};

use rspotify::{
    clients::BaseClient,
    model::{EpisodeId, Id, Market},
    ClientError,
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

/// Path: GET `/episodes/{id}`
pub async fn episode(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let episode_id = EpisodeId::from_id(id.as_str())
        .map_err(|_| ServerError::ParamsError(format!("Invalid episode id: {}", id.as_str())))?;

    let result = account.client.get_an_episode(&episode_id, None).await?;
    json_response(&result)
}

/// Path: GET `/episodes`
pub async fn episodes(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account
        .client
        .get_several_episodes(query.ids().iter(), None)
        .await?;
    json_response(&result)
}

/// Path: GET `/me/episodes`
pub async fn saved_episodes(
    query: web::Query<PageQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let mut limit = query.limit;
    if limit.is_none() {
        limit = Some(50);
    }

    let page = page_saved_episodes(&account, None, limit, query.offset).await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(page))
}

/// Current user saved episodes by page
async fn page_saved_episodes(
    account: &SpotifyAccount,
    market: Option<&Market>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<String, ClientError> {
    let limit = limit.map(|s| s.to_string());
    let offset = offset.map(|s| s.to_string());
    let mut params = HashMap::new();
    if let Some(v) = market.map(|x| x.as_ref()) {
        params.insert("market", v);
    }
    if let Some(v) = limit.as_deref() {
        params.insert("limit", v);
    }
    if let Some(v) = offset.as_deref() {
        params.insert("offset", v);
    }
    account.client.endpoint_get("me/episodes", &params).await
}

/// Path: PUT `/me/episodes`
pub async fn save_episodes(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let mut ids = serde_json::map::Map::new();
    ids.insert("ids".to_string(), query.ids.split(",").collect());

    let result = account
        .client
        .endpoint_put("me/episodes", &serde_json::Value::from(ids))
        .await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(result))
}

/// Path: DELETE `/me/episodes`
pub async fn delete_episodes(
    query: web::Query<IdsQueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let mut ids = serde_json::map::Map::new();
    ids.insert("ids".to_string(), query.ids.split(",").collect());

    let result = account
        .client
        .endpoint_delete("me/episodes", &serde_json::Value::from(ids))
        .await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(result))
}
