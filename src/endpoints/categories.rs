use futures::StreamExt;

use actix_web::{web, HttpResponse};

use rspotify::{
    clients::BaseClient,
    model::{Category, Page},
};

use crate::{
    account::SpotifyAccount,
    app_store::AppStore,
    endpoints::{
        params::{CountryLocateData, LimitOffsetData},
        utils::json_response,
    },
    errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/browse/categories`
/// Get a list of categories used to tag items in Spotify
/// (on, for example, the Spotify player’s “Browse” tab).
#[tracing::instrument(skip(app_store, session))]
pub async fn categories(
    country_locate: web::Query<CountryLocateData>,
    limit_offset: web::Query<LimitOffsetData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    if limit_offset.limit.is_some() {
        let page = page_categories(
            &account,
            country_locate.locale.as_deref(),
            limit_offset.limit,
            limit_offset.offset,
        )
        .await?;
        json_response(&page)
    } else {
        let categories = all_categories(&account, country_locate.locale.as_deref()).await?;
        json_response(&categories)
    }
}

/// All categories
async fn all_categories(
    account: &SpotifyAccount,
    locate: Option<&str>,
) -> Result<Vec<Category>, ServerError> {
    let mut category_stream = account.client.categories(locate, None);
    let mut categories = vec![];
    while let Some(item) = category_stream.next().await {
        if item.is_err() {
            return Err(ServerError::RequestError(format!("{:?}", item)));
        } else {
            categories.push(item.unwrap());
        }
    }
    Ok(categories)
}

/// Album categories by page
async fn page_categories(
    account: &SpotifyAccount,
    locate: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Page<Category>, ServerError> {
    let page = account
        .client
        .categories_manual(locate, None, limit, offset)
        .await?;
    Ok(page)
}
