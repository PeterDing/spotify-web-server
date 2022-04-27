use rspotify::model::{Id, IncludeExternal, Market, SearchType};

#[derive(serde::Deserialize)]
pub struct LoginFormData {
    pub username: String,
    pub password: String,
    // 1: using cache
    // else: no using cache
    pub cache: Option<u8>,
}

#[derive(serde::Deserialize)]
pub struct UserNameQueryData {
    pub username: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SearchQueryData {
    pub q: String,
    #[serde(alias = "type")]
    pub type_: SearchType,
    pub market: Option<Market>,
    pub include_external: Option<IncludeExternal>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Ids Query Data
#[derive(serde::Deserialize)]
pub struct IdsQueryData {
    pub ids: String,
}

impl IdsQueryData {
    pub fn ids<T: Id>(&self) -> Vec<T> {
        self.ids
            .split(',')
            .map(T::from_id)
            .filter(|id| id.is_ok())
            .map(|id| id.unwrap())
            .collect()
    }
}

/// Page Query Data
#[derive(serde::Deserialize)]
pub struct PageQueryData {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
