use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};

mod app_store;
mod errors;
mod routes;
mod session;

const SECRET_KEY: &str = "secret-key--------------------------------------------------++++";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_store = web::Data::new(app_store::AppStore::default());

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::from(SECRET_KEY.as_bytes()),
            ))
            .route("/login", web::post().to(routes::login))
            .route("/search", web::get().to(routes::search))
            .route("/health_check", web::get().to(routes::health_check))
            .app_data(app_store.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
