use actix_web::{http::header::ContentType, web, HttpResponse};

use rspotify::clients::BaseClient;

use crate::{
    app_store::AppStore, endpoints::params::SearchData, errors::ServerError,
    session::ServerSession,
};

pub async fn search(
    query: web::Query<SearchData>,
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
