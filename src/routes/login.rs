use actix_web::{web, HttpResponse};

use crate::{
    account::SpotifyAccount, app_store::AppStore, errors::ServerError, session::ServerSession,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

pub async fn login(
    form: web::Form<FormData>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let account = SpotifyAccount::login(form.username.clone(), form.password.clone()).await?;
    app_store
        .insert_account(form.username.clone(), account)
        .await;
    session.insert_username(&form.username)?;
    Ok(HttpResponse::Ok().finish())
}
