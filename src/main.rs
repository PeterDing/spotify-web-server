use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};

use spotify_web_server::{app_store::AppStore, routes::route};

const SECRET_KEY: &str = "secret-key--------------------------------------------------++++";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_store = web::Data::new(AppStore::default());

    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::from(SECRET_KEY.as_bytes()),
            ))
            .service(route())
            .app_data(app_store.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
