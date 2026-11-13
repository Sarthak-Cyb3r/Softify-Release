use serde::{Deserialize, Serialize};

/// Canonical metadata for a music track exposed across UniFFI.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_seconds: u32,
    pub cover_url: Option<String>,
}

/// Resolved stream audio playback details.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamResult {
    pub stream_url: String,
    pub bitrate: u32,      // 320000 for JioSaavn Studio Master
    pub container: String, // "m4a"
    pub codec: String,     // "aac"
    pub provider: String,  // "jiosaavn_studio_320k"
}

/// Search result collection for query operations.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub tracks: Vec<TrackMetadata>,
    pub provider: String,
}

/// Target audio fidelity levels.
#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioQuality {
    Low96k,
    Medium160k,
    Studio320k,
}
