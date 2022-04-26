use std::future::{ready, Ready};

use actix_session::{Session, SessionExt};
use actix_web::{dev::Payload, FromRequest, HttpRequest};

use crate::{account::UserName, errors::ServerError};

pub struct ServerSession(Session);

impl ServerSession {
    const USERNAME_KEY: &'static str = "username";

    pub fn get_username(&self) -> Result<UserName, ServerError> {
        self.0
            .get(Self::USERNAME_KEY)
            .map_err(|e| ServerError::InnerError(format!("serde error: {:?}", e)))?
            .ok_or(ServerError::NoLoginError)
    }

    pub fn insert_username(&self, username: &str) -> Result<(), ServerError> {
        Ok(self
            .0
            .insert(Self::USERNAME_KEY, username)
            .map_err(|e| ServerError::InnerError(format!("serde error: {:?}", e)))?)
    }

    pub fn log_out(&self) {
        self.0.purge()
    }
}

impl FromRequest for ServerSession {
    type Error = <Session as FromRequest>::Error;
    type Future = Ready<Result<ServerSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(ServerSession(req.get_session())))
    }
}
