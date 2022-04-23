use actix_web::{http::header::ContentType, web, HttpResponse};

use rspotify::{
    clients::BaseClient,
    model::{IncludeExternal, Market, SearchType},
};

use crate::{app_store::AppStore, errors::ServerError, session::ServerSession};

#[derive(serde::Deserialize)]
pub struct QueryData {
    q: String,
    #[serde(alias = "type")]
    type_: SearchType,
    market: Option<Market>,
    include_external: Option<IncludeExternal>,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub async fn search(
    query: web::Query<QueryData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let result = account
        .client
        .search(
            &query.q,
            &query.type_,
            query.market.as_ref(),
            query.include_external.as_ref(),
            query.limit,
            query.offset,
        )
        .await?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::to_string(&result)?))
}
