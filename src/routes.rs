use crate::endpoints::{
    albums, artists, audios, categories, episodes, genres, health_check, login, playlists,
    recommends, search, shows, tracks,
};

use actix_web::web;

pub fn route() -> actix_web::Scope {
    web::scope("")
        .route("/health_check", web::get().to(health_check::health_check))
        // Login api
        .route("/login", web::post().to(login::login))
        .route("/miracle", web::get().to(login::miracle))
        // Search api
        .route("/search", web::get().to(search::search))
        // Albums apis
        .route("/albums/{id}", web::get().to(albums::album))
        .route("/albums", web::get().to(albums::albums))
        .route("/albums/{id}/tracks", web::get().to(albums::album_tracks))
        .route("/me/albums", web::get().to(albums::saved_albums))
        .route("/me/albums", web::put().to(albums::save_albums))
        .route("/me/albums", web::delete().to(albums::delete_albums))
        .route("/browse/new-releases", web::get().to(albums::new_releases))
        // Artists apis
        .route("/artists/{id}", web::get().to(artists::artist))
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
        // Shows apis
        .route("/shows/{id}", web::get().to(shows::show))
        .route("/shows", web::get().to(shows::shows))
        .route("/shows/{id}/episodes", web::get().to(shows::show_episodes))
        .route("/me/shows", web::get().to(shows::saved_shows))
        .route("/me/shows", web::put().to(shows::save_shows))
        .route("/me/shows", web::delete().to(shows::delete_shows))
        // Episodes apis
        .route("/episodes/{id}", web::get().to(episodes::episode))
        .route("/episodes", web::get().to(episodes::episodes))
        .route("/me/episodes", web::get().to(episodes::saved_episodes))
        .route("/me/episodes", web::put().to(episodes::save_episodes))
        .route("/me/episodes", web::delete().to(episodes::delete_episodes))
        // Tracks apis
        .route("/tracks/{id}", web::get().to(tracks::track))
        .route("/tracks", web::get().to(tracks::tracks))
        .route("/me/tracks", web::get().to(tracks::saved_tracks))
        .route("/me/tracks", web::put().to(tracks::save_tracks))
        .route("/me/tracks", web::delete().to(tracks::delete_tracks))
        .route(
            "/audio-features/{id}",
            web::get().to(tracks::track_features),
        )
        .route("/audio-features", web::get().to(tracks::tracks_features))
        .route(
            "/audio-analysis/{id}",
            web::get().to(tracks::track_analysis),
        )
        // Audios apis
        .route("/audio/{id}", web::get().to(audios::audio))
        .route("/audio-stream/{id}", web::get().to(audios::audio_stream))
        // Recommendations
        .route(
            "recommendations",
            web::get().to(recommends::recommendations),
        )
        .route("/playlists/{id}", web::get().to(playlists::playlist))
        .route(
            "/playlists/{id}",
            web::put().to(playlists::change_playlist_detail),
        )
        .route(
            "/playlists/{id}/tracks",
            web::get().to(playlists::playlist_tracks),
        )
        .route(
            "/playlists/{id}/tracks",
            web::post().to(playlists::playlist_add_items),
        )
        .route(
            "/me/playlists",
            web::get().to(playlists::current_user_playlists),
        )
        .route(
            "/users/{id}/playlists",
            web::get().to(playlists::user_playlists),
        )
        .route(
            "/playlists/{id}/followers",
            web::put().to(playlists::follow_playlist),
        )
        .route(
            "/playlists/{id}/followers",
            web::delete().to(playlists::unfollow_playlist),
        )
        .route(
            "/users/{id}/playlists",
            web::post().to(playlists::create_playlist),
        )
        .route(
            "/browse/featured-playlists",
            web::get().to(playlists::featured_playlists),
        )
        .route(
            "/browse/categories/{id}/playlists",
            web::get().to(playlists::category_playlists),
        )
        .route("/browse/categories", web::get().to(categories::categories))
        .route(
            "/recommendations/available-genre-seeds",
            web::get().to(genres::genres),
        )
}
