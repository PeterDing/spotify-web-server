use std::io::{Read, Seek, SeekFrom};

use actix_web::{web, HttpResponse};

use librespot::{
    audio::{AudioDecrypt, AudioFile},
    core::spotify_id::SpotifyId,
    metadata::{AudioItem, FileFormat},
};

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use crate::{
    account::{SpotifyAccount, UserName},
    app_store::AppStore,
    common::hex,
    endpoints::utils::ok_with_body_response,
    errors::ServerError,
    session::ServerSession,
};

/// Path: GET `/audio/{id}`
/// Audio files information
#[tracing::instrument(skip(app_store, session))]
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct UserNameTrackId {
    username: String,
    track_id: String,
}

/// Path: GET `/audio-uri/{id}`
/// Audio direct uri which returns a uri to the track audio stream
#[tracing::instrument(skip(app_store, session))]
pub async fn audio_uri(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username.clone()).await?;

    let audio_sign = UserNameTrackId {
        username: username.as_ref().to_owned(),
        track_id: id.to_string(),
    };

    let (enc, iv) = account.encrypt(serde_json::to_string(&audio_sign)?.as_bytes());

    let url = format!(
        "/audio-stream-with-sign?sign={}&iv={}&username={}",
        hex::encode(&enc),
        hex::encode(&iv),
        utf8_percent_encode(username.as_ref(), NON_ALPHANUMERIC),
    );

    ok_with_body_response(url)
}

#[derive(Debug, serde::Deserialize)]
pub struct AudioSign {
    sign: String,
    iv: String,
    username: String,
}

/// Path: GET `/audio-stream-with-sign/{id}`
/// The track audio stream with sign parameters
#[tracing::instrument(skip(app_store))]
pub async fn audio_stream_with_sign(
    audio_sign: web::Query<AudioSign>,
    app_store: web::Data<AppStore>,
) -> Result<HttpResponse, ServerError> {
    let iv = hex::decode(audio_sign.iv.as_str())?;
    let sign = hex::decode(audio_sign.sign.as_str())?;

    let username: UserName = audio_sign.username.as_str().into();
    let account = app_store.authorize(username).await?;

    let dec = account.decrypt(&iv, &sign)?;
    match serde_json::from_slice::<UserNameTrackId>(&dec) {
        Ok(username_trackid) => audio_cn_stream(&username_trackid.track_id, &account).await,
        Err(_) => Err(ServerError::AuthenticationError),
    }
}

/// Path: GET `/audio-stream/{id}`
/// The track audio stream without sign but needs Cookies
#[tracing::instrument(skip(app_store, session))]
pub async fn audio_stream(
    id: web::Path<String>,
    app_store: web::Data<AppStore>,
    session: ServerSession,
) -> Result<HttpResponse, ServerError> {
    let username = session.get_username()?;
    let account = app_store.authorize(username).await?;

    audio_cn_stream(id.as_str(), &account).await
}

/// Audio content stream
async fn audio_cn_stream(id: &str, account: &SpotifyAccount) -> Result<HttpResponse, ServerError> {
    let spotify_id = SpotifyId::from_uri(&format!("spotify:track:{}", id))
        .map_err(|_| ServerError::ParamsError(format!("Track id {} is invalid", id)))?;

    let audio_item = AudioItem::get_audio_item(&account.session, spotify_id).await?;

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

    let size = 1024 * 10;
    let mut buf = Vec::with_capacity(size);
    unsafe {
        buf.set_len(size);
    }
    let s = async_stream::stream! {
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
