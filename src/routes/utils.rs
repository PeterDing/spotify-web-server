use actix_web::{http::header::ContentType, HttpResponse};

use crate::errors::ServerError;

pub fn ok_response() -> Result<HttpResponse, ServerError> {
    Ok(HttpResponse::Ok().finish())
}

pub fn json_response(obj: impl serde::Serialize) -> Result<HttpResponse, ServerError> {
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(serde_json::to_string(&obj)?))
}
