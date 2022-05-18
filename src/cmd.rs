use rand::RngCore;
use sha2::Digest;
use url::Url;

use crate::app_store::DEFAULT_CLIENT_ID;

#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cmd {
    #[clap(long, default_value_t = String::from(DEFAULT_CLIENT_ID), help = "Spotify client ID")]
    pub client_id: String,

    #[clap(long, default_value_t = String::from("127.0.0.1"), help = "Server bind address")]
    pub bind: String,

    #[clap(long, default_value_t = 0, help = "Server listen port")]
    pub port: u16,

    #[clap(long, help = "Session secret")]
    pub session_secret: Option<String>,

    #[clap(long, help = "Load cached authentication when server starts")]
    pub load_cache: bool,

    #[clap(long, default_value_t = String::from("~/.spotify-web-server/authentication"), help = "Cache directory")]
    pub cache_dir: String,

    #[clap(long, default_value_t = String::from("info"), help = "Cache directory")]
    pub log_level: String,

    #[clap(long, help = "Proxy url")]
    pub proxy: Option<Url>,
}

impl Cmd {
    pub fn session_secret(&self) -> [u8; 64] {
        let mut result = [0u8; 64];
        if let Some(secret) = &self.session_secret {
            let mut hasher = sha2::Sha512::new();
            hasher.update(secret.as_bytes());
            let output = hasher.finalize();
            result.copy_from_slice(&output[..]);
        } else {
            let mut rand_array = [0u8; 64];
            rand::thread_rng().fill_bytes(&mut rand_array);
            let mut hasher = sha2::Sha256::new();
            hasher.update(&rand_array);
            let output = hasher.finalize();
            result.copy_from_slice(&output[..]);
        }

        result
    }
}
