mod albums;
mod auth;
mod health_check;
mod login;
mod search;
mod utils;

pub use health_check::health_check;
pub use login::login;
pub use search::search;

use actix_web::web;

pub fn route() -> actix_web::Scope {
    web::scope("")
        .route("/health_check", web::get().to(health_check))
        .route("/login", web::post().to(login))
        .route("/search", web::get().to(search))
        .route("/albums", web::get().to(albums::albums))
        .route("/albums/{id}/tracks", web::get().to(albums::album_tracks))
        .route("/me/albums", web::get().to(albums::saved_albums))
        .route("/me/albums", web::put().to(albums::save_albums))
        .route("/me/albums", web::delete().to(albums::delete_albums))
        .route(
            "/browse/new-releases",
            web::delete().to(albums::new_releases),
        )
}
