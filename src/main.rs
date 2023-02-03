use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{Key, SameSite},
    web, App, HttpServer,
};
use clap::Parser;
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use spotify_web_server::{app_store::AppStore, cmd::Cmd, routes::route};

async fn async_main() -> std::io::Result<()> {
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
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(&session_secret),
                )
                .cookie_secure(false)
                .cookie_same_site(SameSite::None)
                .cookie_http_only(false)
                .cookie_domain(None)
                .build(),
            )
            .service(route())
            .app_data(app_store.clone())
    })
    .bind((cmd.bind.as_str(), cmd.port))?
    .run()
    .await
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())?;
    Ok(())
}
