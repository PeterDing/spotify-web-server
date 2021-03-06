use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};
use clap::Parser;
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use spotify_web_server::{app_store::AppStore, cmd::Cmd, routes::route};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cmd = Cmd::parse();
    let cache_dir = cmd.cache_dir.clone();

    let bunyan_formatting_layer =
        BunyanFormattingLayer::new(env!("CARGO_PKG_NAME").to_string(), std::io::stdout);
    let subscriber = Registry::default()
        .with(EnvFilter::new(&cmd.log_level))
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let app_store = AppStore::new(&cmd.client_id, &cache_dir, cmd.proxy.clone());
    if cmd.load_cache {
        app_store.load_cache().await.expect("Failed to load cache");
    };
    let app_store = web::Data::new(app_store);

    let session_secret = cmd.session_secret();
    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                Key::from(&session_secret),
            ))
            .service(route())
            .app_data(app_store.clone())
    })
    .bind((cmd.bind.as_str(), cmd.port))?
    .run()
    .await
}
