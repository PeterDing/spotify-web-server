use actix_web::{http::StatusCode, ResponseError};

use librespot::core::{mercury::MercuryError, session::SessionError};

use rspotify::ClientError;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Spotify Authentication Error")]
    AuthenticationError,
    #[error("No Login Error")]
    NoLoginError,
    #[error("Inner Error: {0}")]
    InnerError(String),
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Spotify Client Error: {0}")]
    ClientError(#[from] ClientError),
    #[error("Spotify Request Error: {0}")]
    RequestError(String),
    #[error("Spotify Session Error: {0}")]
    SessionError(#[from] SessionError),
    #[error("Spotify Token Error")]
    TokenError(MercuryError),
    #[error("Params Error: {0}")]
    ParamsError(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
}

impl From<MercuryError> for ServerError {
    fn from(error: MercuryError) -> Self {
        ServerError::TokenError(error)
    }
}

impl ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::AuthenticationError => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
