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

/// User or curated playlist metadata.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub track_count: u32,
    pub created_at: i64,
}

/// Plain and synchronized lyrics container.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lyrics {
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
    pub is_romanized: bool,
}

/// Listening summary and Wrapped stats calculated from playback history.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrappedStats {
    pub total_play_time_ms: u64,
    pub top_tracks: Vec<TrackMetadata>,
    pub top_artists: Vec<String>,
    pub most_played_track: Option<TrackMetadata>,
    pub unique_artists_count: u32,
    pub favorite_listening_hour: u8,
}

/// Curated moods for algorithmically generated mood mixes.
#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    Chill,
    Workout,
    Focus,
    Romance,
    Party,
    Melancholy,
}

/// Algorithmic mix category variant.
#[derive(uniffi::Enum, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MixType {
    DailyMix1,
    DailyMix2,
    DailyMix3,
    DailyMix4,
    DailyMix5,
    DailyMix6,
    WeeklyDiscovery,
    Mood { mood: Mood },
}

impl MixType {
    pub fn mood(mood: Mood) -> Self {
        MixType::Mood { mood }
    }
}

/// Personalized algorithmic playlist / daily mix.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mix {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub mix_type: MixType,
    pub tracks: Vec<TrackMetadata>,
}

/// 10-band equalizer frequencies: 31Hz, 63Hz, 125Hz, 250Hz, 500Hz, 1kHz, 2kHz, 4kHz, 8kHz, 16kHz.
pub const EQUALIZER_FREQUENCIES: [&str; 10] = [
    "31Hz", "63Hz", "125Hz", "250Hz", "500Hz", "1kHz", "2kHz", "4kHz", "8kHz", "16kHz",
];

/// Audio equalizer preset configuration with 10 frequency band gains in dB.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EqualizerPreset {
    pub name: String,
    pub preamp_db: f32,
    pub bands: Vec<f32>,
}

impl EqualizerPreset {
    /// Standard flat equalizer preset.
    pub fn flat() -> Self {
        Self {
            name: "Flat".to_string(),
            preamp_db: 0.0,
            bands: vec![0.0; 10],
        }
    }

    /// Bass boost preset enhancing lower frequencies (31Hz - 250Hz).
    pub fn bass_boost() -> Self {
        Self {
            name: "Bass Boost".to_string(),
            preamp_db: 0.0,
            bands: vec![6.0, 5.5, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    /// Vocal boost preset enhancing mid-range vocal frequencies.
    pub fn vocal_boost() -> Self {
        Self {
            name: "Vocal Boost".to_string(),
            preamp_db: 0.0,
            bands: vec![-1.0, -0.5, 0.0, 2.0, 3.5, 3.5, 2.0, 1.0, 0.0, 0.0],
        }
    }

    /// Validates that the preset has exactly 10 bands.
    pub fn is_valid(&self) -> bool {
        self.bands.len() == 10
    }
}

/// Application version update and release details.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub current_version: String,
    pub has_update: bool,
    pub release_notes: Option<String>,
    pub download_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_metadata_serialization() {
        let track = TrackMetadata {
            id: "track_123".to_string(),
            title: "Blinding Lights".to_string(),
            artist: "The Weeknd".to_string(),
            album: Some("After Hours".to_string()),
            duration_seconds: 200,
            cover_url: Some("https://example.com/cover.jpg".to_string()),
        };

        let json = serde_json::to_string(&track).unwrap();
        let decoded: TrackMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(track, decoded);
    }

    #[test]
    fn test_stream_result_serialization() {
        let stream = StreamResult {
            stream_url: "https://aac.saavncdn.com/sample_320.mp4".to_string(),
            bitrate: 320000,
            container: "m4a".to_string(),
            codec: "aac".to_string(),
            provider: "jiosaavn_studio_320k".to_string(),
        };

        let json = serde_json::to_string(&stream).unwrap();
        let decoded: StreamResult = serde_json::from_str(&json).unwrap();
        assert_eq!(stream, decoded);
    }

    #[test]
    fn test_playlist_serialization() {
        let playlist = Playlist {
            id: "pl_1".to_string(),
            title: "Workout Bops".to_string(),
            description: Some("High energy tracks".to_string()),
            cover_url: None,
            track_count: 24,
            created_at: 1700000000,
        };

        let json = serde_json::to_string(&playlist).unwrap();
        let decoded: Playlist = serde_json::from_str(&json).unwrap();
        assert_eq!(playlist, decoded);
    }

    #[test]
    fn test_lyrics_serialization() {
        let lyrics = Lyrics {
            plain_lyrics: Some("I said, ooh, I'm blinded by the lights".to_string()),
            synced_lyrics: Some("[00:15.20] I said, ooh, I'm blinded by the lights".to_string()),
            is_romanized: false,
        };

        let json = serde_json::to_string(&lyrics).unwrap();
        let decoded: Lyrics = serde_json::from_str(&json).unwrap();
        assert_eq!(lyrics, decoded);
    }

    #[test]
    fn test_wrapped_stats_serialization() {
        let track = TrackMetadata {
            id: "t1".to_string(),
            title: "Song".to_string(),
            artist: "Artist".to_string(),
            album: None,
            duration_seconds: 180,
            cover_url: None,
        };

        let stats = WrappedStats {
            total_play_time_ms: 36000000,
            top_tracks: vec![track.clone()],
            top_artists: vec!["Artist".to_string()],
            most_played_track: Some(track),
            unique_artists_count: 42,
            favorite_listening_hour: 22,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let decoded: WrappedStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, decoded);
    }

    #[test]
    fn test_mix_and_mix_types() {
        let mix_daily = MixType::DailyMix1;
        let mix_mood = MixType::mood(Mood::Chill);

        assert_eq!(mix_daily, MixType::DailyMix1);
        assert_eq!(mix_mood, MixType::Mood { mood: Mood::Chill });

        let mix = Mix {
            id: "mix_chill".to_string(),
            title: "Chill Evening Mix".to_string(),
            description: Some("Relaxing tunes".to_string()),
            mix_type: mix_mood,
            tracks: vec![],
        };

        let json = serde_json::to_string(&mix).unwrap();
        let decoded: Mix = serde_json::from_str(&json).unwrap();
        assert_eq!(mix, decoded);
    }

    #[test]
    fn test_equalizer_presets() {
        let flat = EqualizerPreset::flat();
        assert_eq!(flat.bands.len(), 10);
        assert!(flat.is_valid());

        let bass = EqualizerPreset::bass_boost();
        assert_eq!(bass.bands.len(), 10);
        assert!(bass.is_valid());
        assert_eq!(bass.bands[0], 6.0);

        let json = serde_json::to_string(&bass).unwrap();
        let decoded: EqualizerPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(bass, decoded);
    }

    #[test]
    fn test_update_info() {
        let update = UpdateInfo {
            latest_version: "1.2.0".to_string(),
            current_version: "1.1.0".to_string(),
            has_update: true,
            release_notes: Some("Added studio 320k equalizer and mixes".to_string()),
            download_url: Some("https://github.com/org/repo/releases/tag/v1.2.0".to_string()),
        };

        let json = serde_json::to_string(&update).unwrap();
        let decoded: UpdateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(update, decoded);
    }
}
