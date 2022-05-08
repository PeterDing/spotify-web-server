use actix_web::HttpResponse;

use crate::{errors::ServerError, session::ServerSession};

#[tracing::instrument(skip(session))]
pub async fn health_check(session: ServerSession) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let body = username.as_ref().to_string();
    Ok(HttpResponse::Ok().body(body))
}
