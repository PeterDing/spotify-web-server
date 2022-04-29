use rspotify::model::{Id, IncludeExternal, Market, RecommendationsAttribute, SearchType};

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

pub fn into_ids<T: Id>(s: &str) -> Vec<T> {
    s.split(',')
        .map(T::from_id)
        .filter(|id| id.is_ok())
        .map(|id| id.unwrap())
        .collect()
}

/// Ids Query Data
#[derive(serde::Deserialize)]
pub struct IdsQueryData {
    pub ids: String,
}

impl IdsQueryData {
    pub fn ids<T: Id>(&self) -> Vec<T> {
        into_ids(&self.ids)
    }
}

/// Page Query Data
#[derive(serde::Deserialize)]
pub struct PageQueryData {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Recommendations Query Data
#[derive(serde::Deserialize)]
pub struct RecommendationsQueryData {
    seed_artists: Option<String>,
    seed_genres: Option<String>,
    seed_tracks: Option<String>,
    limit: Option<u32>,
    // Attributes
    min_acousticness: Option<f32>,
    max_acousticness: Option<f32>,
    target_acousticness: Option<f32>,
    min_danceability: Option<f32>,
    max_danceability: Option<f32>,
    target_danceability: Option<f32>,
    min_duration_ms: Option<i32>,
    max_duration_ms: Option<i32>,
    target_duration_ms: Option<i32>,
    min_energy: Option<f32>,
    max_energy: Option<f32>,
    target_energy: Option<f32>,
    min_instrumentalness: Option<f32>,
    max_instrumentalness: Option<f32>,
    target_instrumentalness: Option<f32>,
    min_key: Option<i32>,
    max_key: Option<i32>,
    target_key: Option<i32>,
    min_liveness: Option<f32>,
    max_liveness: Option<f32>,
    target_liveness: Option<f32>,
    min_loudness: Option<f32>,
    max_loudness: Option<f32>,
    target_loudness: Option<f32>,
    min_mode: Option<i32>,
    max_mode: Option<i32>,
    target_mode: Option<i32>,
    min_popularity: Option<i32>,
    max_popularity: Option<i32>,
    target_popularity: Option<i32>,
    min_speechiness: Option<f32>,
    max_speechiness: Option<f32>,
    target_speechiness: Option<f32>,
    min_tempo: Option<f32>,
    max_tempo: Option<f32>,
    target_tempo: Option<f32>,
    min_time_signature: Option<i32>,
    max_time_signature: Option<i32>,
    target_time_signature: Option<i32>,
    min_valence: Option<f32>,
    max_valence: Option<f32>,
    target_valence: Option<f32>,
}

impl RecommendationsQueryData {
    pub fn seed_artists<T: Id>(&self) -> Option<Vec<T>> {
        self.seed_artists.as_ref().map(|sa| into_ids(sa))
    }

    pub fn seed_genres(&self) -> Option<Vec<&str>> {
        self.seed_genres.as_ref().map(|sa| sa.split(',').collect())
    }

    pub fn seed_tracks<T: Id>(&self) -> Option<Vec<T>> {
        self.seed_tracks.as_ref().map(|sa| into_ids(sa))
    }

    pub fn limit(&self) -> Option<u32> {
        self.limit
    }

    pub fn attributes(&self) -> Vec<RecommendationsAttribute> {
        let mut attributes = vec![];
        if let Some(val) = self.min_acousticness {
            attributes.push(RecommendationsAttribute::MinAcousticness(val));
        }
        if let Some(val) = self.max_acousticness {
            attributes.push(RecommendationsAttribute::MaxAcousticness(val));
        }
        if let Some(val) = self.target_acousticness {
            attributes.push(RecommendationsAttribute::TargetAcousticness(val));
        }
        if let Some(val) = self.min_danceability {
            attributes.push(RecommendationsAttribute::MinDanceability(val));
        }
        if let Some(val) = self.max_danceability {
            attributes.push(RecommendationsAttribute::MaxDanceability(val));
        }
        if let Some(val) = self.target_danceability {
            attributes.push(RecommendationsAttribute::TargetDanceability(val));
        }
        if let Some(val) = self.min_duration_ms {
            attributes.push(RecommendationsAttribute::MinDurationMs(val));
        }
        if let Some(val) = self.max_duration_ms {
            attributes.push(RecommendationsAttribute::MaxDurationMs(val));
        }
        if let Some(val) = self.target_duration_ms {
            attributes.push(RecommendationsAttribute::TargetDurationMs(val));
        }
        if let Some(val) = self.min_energy {
            attributes.push(RecommendationsAttribute::MinEnergy(val));
        }
        if let Some(val) = self.max_energy {
            attributes.push(RecommendationsAttribute::MaxEnergy(val));
        }
        if let Some(val) = self.target_energy {
            attributes.push(RecommendationsAttribute::TargetEnergy(val));
        }
        if let Some(val) = self.min_instrumentalness {
            attributes.push(RecommendationsAttribute::MinInstrumentalness(val));
        }
        if let Some(val) = self.max_instrumentalness {
            attributes.push(RecommendationsAttribute::MaxInstrumentalness(val));
        }
        if let Some(val) = self.target_instrumentalness {
            attributes.push(RecommendationsAttribute::TargetInstrumentalness(val));
        }
        if let Some(val) = self.min_key {
            attributes.push(RecommendationsAttribute::MinKey(val));
        }
        if let Some(val) = self.max_key {
            attributes.push(RecommendationsAttribute::MaxKey(val));
        }
        if let Some(val) = self.target_key {
            attributes.push(RecommendationsAttribute::TargetKey(val));
        }
        if let Some(val) = self.min_liveness {
            attributes.push(RecommendationsAttribute::MinLiveness(val));
        }
        if let Some(val) = self.max_liveness {
            attributes.push(RecommendationsAttribute::MaxLiveness(val));
        }
        if let Some(val) = self.target_liveness {
            attributes.push(RecommendationsAttribute::TargetLiveness(val));
        }
        if let Some(val) = self.min_loudness {
            attributes.push(RecommendationsAttribute::MinLoudness(val));
        }
        if let Some(val) = self.max_loudness {
            attributes.push(RecommendationsAttribute::MaxLoudness(val));
        }
        if let Some(val) = self.target_loudness {
            attributes.push(RecommendationsAttribute::TargetLoudness(val));
        }
        if let Some(val) = self.min_mode {
            attributes.push(RecommendationsAttribute::MinMode(val));
        }
        if let Some(val) = self.max_mode {
            attributes.push(RecommendationsAttribute::MaxMode(val));
        }
        if let Some(val) = self.target_mode {
            attributes.push(RecommendationsAttribute::TargetMode(val));
        }
        if let Some(val) = self.min_popularity {
            attributes.push(RecommendationsAttribute::MinPopularity(val));
        }
        if let Some(val) = self.max_popularity {
            attributes.push(RecommendationsAttribute::MaxPopularity(val));
        }
        if let Some(val) = self.target_popularity {
            attributes.push(RecommendationsAttribute::TargetPopularity(val));
        }
        if let Some(val) = self.min_speechiness {
            attributes.push(RecommendationsAttribute::MinSpeechiness(val));
        }
        if let Some(val) = self.max_speechiness {
            attributes.push(RecommendationsAttribute::MaxSpeechiness(val));
        }
        if let Some(val) = self.target_speechiness {
            attributes.push(RecommendationsAttribute::TargetSpeechiness(val));
        }
        if let Some(val) = self.min_tempo {
            attributes.push(RecommendationsAttribute::MinTempo(val));
        }
        if let Some(val) = self.max_tempo {
            attributes.push(RecommendationsAttribute::MaxTempo(val));
        }
        if let Some(val) = self.target_tempo {
            attributes.push(RecommendationsAttribute::TargetTempo(val));
        }
        if let Some(val) = self.min_time_signature {
            attributes.push(RecommendationsAttribute::MinTimeSignature(val));
        }
        if let Some(val) = self.max_time_signature {
            attributes.push(RecommendationsAttribute::MaxTimeSignature(val));
        }
        if let Some(val) = self.target_time_signature {
            attributes.push(RecommendationsAttribute::TargetTimeSignature(val));
        }
        if let Some(val) = self.min_valence {
            attributes.push(RecommendationsAttribute::MinValence(val));
        }
        if let Some(val) = self.max_valence {
            attributes.push(RecommendationsAttribute::MaxValence(val));
        }
        if let Some(val) = self.target_valence {
            attributes.push(RecommendationsAttribute::TargetValence(val));
        }

        attributes
    }
}
