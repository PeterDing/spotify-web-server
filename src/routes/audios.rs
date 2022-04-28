use core::task::{Context, Poll};
use std::{
    collections::HashSet,
    io::{Read, Seek, SeekFrom},
    pin::Pin,
};

use futures::{Stream, StreamExt};

use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};

use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{Id, Page, SavedTrack, TrackId},
};

use librespot::{
    audio::{AudioDecrypt, AudioFile},
    core::{
        authentication::Credentials, cache::Cache, config::SessionConfig, keymaster,
        session::Session, spotify_id::SpotifyId,
    },
    metadata::{Album, AudioItem, FileFormat, Metadata},
};

use crate::{
    account::SpotifyAccount,
    app_store::AppStore,
    errors::ServerError,
    routes::{
        params::{IdsQueryData, PageQueryData},
        utils::{json_response, ok_response},
    },
    session::ServerSession,
};

use super::utils::ok_with_body_response;

/// Path: GET `/audio/{id}`
pub async fn audio(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let spotify_id = SpotifyId::from_uri(&format!("spotify:track:{}", id.as_str()))
        .map_err(|_| ServerError::ParamsError(format!("Track id {} is invalid", id.as_str())))?;

    let result = AudioItem::get_audio_item(&account.session, spotify_id).await?;

    ok_with_body_response(format!("{:?}", result))
}

/// Path: GET `/audio-stream/{id}`
pub async fn audio_stream(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    let spotify_id = SpotifyId::from_uri(&format!("spotify:track:{}", id.as_str()))
        .map_err(|_| ServerError::ParamsError(format!("Track id {} is invalid", id.as_str())))?;

    let audio_item = AudioItem::get_audio_item(&account.session, spotify_id).await?;

    println!("Hello, world! {:?}", audio_item);

    // let file_id = audio_item.files.get(&FileFormat::OGG_VORBIS_96).unwrap();
    let file_id = audio_item.files.get(&FileFormat::OGG_VORBIS_320).unwrap();

    let key = account
        .session
        .audio_key()
        .request(spotify_id, *file_id)
        .await
        .expect("audio key failed");

    let enc_file = AudioFile::open(&account.session, *file_id, 500 * 1024, true)
        .await
        .unwrap();

    let stream_loader_controller = enc_file.get_stream_loader_controller();
    stream_loader_controller.set_stream_mode();

    let mut decrypted_file = AudioDecrypt::new(key, enc_file);
    decrypted_file.seek(SeekFrom::Start(0xa7)).unwrap();

    let s = async_stream::stream! {
        let mut buf = [0u8; 1024 * 2];
        loop {
            let n = decrypted_file.read(&mut buf);
            match n {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    yield Ok(web::BytesMut::from(&buf[..n]).freeze());
                }
                Err(e) => yield Err(e),
            }
        }
    };

    Ok(HttpResponse::Ok().streaming(s))
}
