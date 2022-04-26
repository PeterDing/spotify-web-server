mod albums;
mod artists;
mod auth;
mod health_check;
mod login;
mod search;
mod utils;

pub use health_check::health_check;
pub use login::{login, miracle};
pub use search::search;

use actix_web::web;

pub fn route() -> actix_web::Scope {
    web::scope("")
        .route("/health_check", web::get().to(health_check))
        // Login api
        .route("/login", web::post().to(login))
        .route("/miracle", web::get().to(miracle))
        // Search api
        .route("/search", web::get().to(search))
        // Albums apis
        .route("/albums", web::get().to(albums::albums))
        .route("/albums/{id}/tracks", web::get().to(albums::album_tracks))
        .route("/me/albums", web::get().to(albums::saved_albums))
        .route("/me/albums", web::put().to(albums::save_albums))
        .route("/me/albums", web::delete().to(albums::delete_albums))
        .route("/browse/new-releases", web::get().to(albums::new_releases))
        // Artists apis
        .route("/artists", web::get().to(artists::artists))
        .route(
            "/artists/{id}/albums",
            web::get().to(artists::artist_albums),
        )
        .route(
            "/artists/{id}/top-tracks",
            web::get().to(artists::artist_top_tracks),
        )
        .route(
            "/artists/{id}/related-artists",
            web::get().to(artists::artist_related_artists),
        )
}
