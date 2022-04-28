use std::path::Path;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};

use librespot::core::cache::Cache;

use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use spotify_web_server::{
    account::{
        utils::{load_credentials, CONFIG_ROOT},
        SpotifyAccount,
    },
    app_store::AppStore,
    routes::route,
};

const SECRET_KEY: &str = "secret-key--------------------------------------------------++++";

async fn init_app_store() -> AppStore {
    let app_store = AppStore::default();

    let config_dir = Path::new(CONFIG_ROOT);
    for entry in config_dir.read_dir().expect("No config directory") {
        if let Ok(entry) = entry {
            let config_dir = entry.path();
            if let Some(credentials) = load_credentials(config_dir.clone()) {
                let username = config_dir.file_name().unwrap().to_str().unwrap();
                let cache =
                    Cache::new(Some(config_dir.clone()), None, None).expect("fail config path");
                let account = SpotifyAccount::new(credentials, cache)
                    .await
                    .expect("Cache auth failed");
                app_store.insert_account(username, account).await;
            }
        }
    }

    app_store
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bunyan_formatting_layer =
        BunyanFormattingLayer::new(env!("CARGO_PKG_NAME").to_string(), std::io::stdout);
    let subscriber = Registry::default()
        .with(EnvFilter::new("info"))
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let app_store = web::Data::new(init_app_store().await);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::from(SECRET_KEY.as_bytes()),
            ))
            .service(route())
            .app_data(app_store.clone())
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
