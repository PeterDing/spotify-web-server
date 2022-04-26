use std::path::Path;

use librespot::core::{authentication::Credentials, cache::Cache};

pub const CONFIG_ROOT: &str = "./config";

pub fn load_credentials<P: AsRef<Path>>(path: P) -> Option<Credentials> {
    if let Ok(cache) = Cache::new(Some(path), None, None) {
        cache.credentials()
    } else {
        None
    }
}
