use std::{
    io::{Read, Seek, SeekFrom},
    time::Duration,
};

use actix_web::{web, HttpResponse};
use tokio::time::timeout;
use tokio_stream::StreamExt;

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

    let account_session = &account.session.read().await;
    let result = AudioItem::get_audio_item(&account_session, spotify_id).await?;

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
        "/audio-stream-with-sign/audio.ogg?sign={}&iv={}&username={}",
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
        Ok(username_trackid) => {
            // retry_audio_cn_stream(&username_trackid.track_id, &account, 3).await
            audio_cn_stream(&username_trackid.track_id, &account).await
        }
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

    // retry_audio_cn_stream(id.as_str(), &account, 3).await
    audio_cn_stream(id.as_str(), &account).await
}

/// Audio content stream
#[tracing::instrument(skip(account))]
async fn audio_cn_without_stream(
    id: &str,
    account: &SpotifyAccount,
) -> Result<HttpResponse, ServerError> {
    println!("------------ audio_cn_without_stream start");
    let spotify_id = SpotifyId::from_uri(&format!("spotify:track:{}", id))
        .map_err(|_| ServerError::ParamsError(format!("Track id {} is invalid", id)))?;

    let account_session = &account.session.read().await;
    println!("------------ audio_cn_without_stream: Gotten account session");
    tracing::info!("Gotten account session");

    let audio_item = AudioItem::get_audio_item(&account_session, spotify_id).await?;
    println!("------------ audio_cn_without_stream: Gotten audio item");
    tracing::info!("Gotten audio item");

    let file_id = audio_item.files.get(&FileFormat::OGG_VORBIS_320).unwrap();
    println!("------------ audio_cn_without_stream: Gotten audio file");
    tracing::info!(
        "Audio file id: {}",
        file_id
            .to_base16()
            .unwrap_or("file_id decode failed".to_owned())
    );

    let enc_file = AudioFile::open(&account_session, *file_id, 500 * 1024, true).await?;
    println!("------------ audio_cn_without_stream: Gotten encrypt file");
    tracing::info!("Gotten encrypt file");

    let stream_loader_controller = enc_file.get_stream_loader_controller();
    stream_loader_controller.set_stream_mode();

    let key = account_session
        .audio_key()
        .request(spotify_id, *file_id)
        .await?;

    println!("------------ audio_cn_without_stream: Gotten audio key");
    tracing::info!("Gotten audio key: {:?}", key);

    let mut decrypted_file = AudioDecrypt::new(key, enc_file);
    decrypted_file.seek(SeekFrom::Start(0xa7))?;

    let size = 1024 * 10;
    let mut buf = vec![0u8; size];
    match decrypted_file.read_to_end(&mut buf) {
        Ok(n) => {
            println!("------------ audio_cn_without_stream: Start audio stream");
            tracing::info!("Start audio stream");
            return Ok(HttpResponse::Ok().content_type("audio/ogg").body(buf));
        }
        Err(e) => {
            println!("------------ audio_cn_without_stream: stream timeout");
            tracing::warn!("stream data is timeout");
            return Err(ServerError::AudioError(format!("{:?}", e)));
        }
    }
}

/// Audio content stream
#[tracing::instrument(skip(account))]
async fn audio_cn_stream(id: &str, account: &SpotifyAccount) -> Result<HttpResponse, ServerError> {
    println!("------------ audio_cn_stream start");
    let spotify_id = SpotifyId::from_uri(&format!("spotify:track:{}", id))
        .map_err(|_| ServerError::ParamsError(format!("Track id {} is invalid", id)))?;

    let account_session = &account.session.read().await;
    println!("------------ audio_cn_stream: Gotten account session");
    tracing::info!("Gotten account session");

    let audio_item = AudioItem::get_audio_item(&account_session, spotify_id).await?;
    println!("------------ audio_cn_stream: Gotten audio item");
    tracing::info!("Gotten audio item");

    let file_id = audio_item.files.get(&FileFormat::OGG_VORBIS_320).unwrap();
    println!("------------ audio_cn_stream: Gotten audio file");
    tracing::info!(
        "Audio file id: {}",
        file_id
            .to_base16()
            .unwrap_or("file_id decode failed".to_owned())
    );

    let enc_file = AudioFile::open(&account_session, *file_id, 500 * 1024, true).await?;
    println!("------------ audio_cn_stream: Gotten encrypt file");
    tracing::info!("Gotten encrypt file");

    let stream_loader_controller = enc_file.get_stream_loader_controller();
    stream_loader_controller.set_stream_mode();

    let key = account_session
        .audio_key()
        .request(spotify_id, *file_id)
        .await?;

    println!("------------ audio_cn_stream: Gotten audio key");
    tracing::info!("Gotten audio key: {:?}", key);

    let mut decrypted_file = AudioDecrypt::new(key, enc_file);
    decrypted_file.seek(SeekFrom::Start(0xa7))?;

    let size = 1024 * 10;
    let mut buf = vec![0u8; size];
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
                Err(e) => {
                    yield Err(e);
                    drop(decrypted_file);
                    break;
                },
            }
        }
    }
    .timeout(Duration::from_millis(100))
    .take_while(|r| {
        if r.is_err() {
            println!("------------ audio_cn_stream: stream timeout");
            tracing::warn!("stream data is timeout");
        }
        r.is_ok()
    })
    .map(|d| d.unwrap());

    println!("------------ audio_cn_stream: Start audio stream");
    tracing::info!("Start audio stream");

    Ok(HttpResponse::Ok().content_type("audio/ogg").streaming(s))
}

/// Retry to get audio content stream
#[tracing::instrument(skip(account))]
async fn retry_audio_cn_stream(
    id: &str,
    account: &SpotifyAccount,
    retries: usize,
) -> Result<HttpResponse, ServerError> {
    for i in 0..retries {
        if i > 0 {
            tracing::warn!("Retry get audio stream by {}", i);
        }

        match timeout(Duration::from_secs(3), audio_cn_stream(id, account)).await {
            Ok(result) => return result,
            Err(err) => {
                // Reset session
                // account.reset_session().await;
                if i + 1 == retries {
                    return Err(ServerError::AudioError(format!(
                        "Failed after {} retries. the last error was: {:?}",
                        retries, err
                    )));
                }
            }
        }
    }

    Err(ServerError::AudioError(format!(
        "Failed after {} retries",
        retries
    )))
}
