//! Listening analytics, telemetry aggregation, and Softify Wrapped engine.

use softify_models::{TrackMetadata, WrappedStats};
use softify_storage::Database;

/// Analytics and Wrapped calculation engine for Softify Next-Gen.
#[derive(Clone)]
pub struct AnalyticsEngine {
    db: Database,
}

impl AnalyticsEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Computes full Softify Wrapped statistics from playback history.
    pub fn compute_wrapped(&self, since_epoch: Option<i64>) -> Result<WrappedStats, rusqlite::Error> {
        self.db.get_wrapped_stats(since_epoch)
    }

    /// Resolves audio persona title from listening metrics.
    pub fn determine_audio_persona(
        total_minutes: u64,
        fav_hour: u8,
        top_artist_plays: usize,
        total_plays: usize,
        unique_artists: usize,
    ) -> String {
        if total_plays == 0 {
            return "The Quiet Observer".to_string();
        }

        if fav_hour >= 22 || fav_hour <= 4 {
            "The Night Owl".to_string()
        } else if (5..=8).contains(&fav_hour) {
            "The Early Bird".to_string()
        } else if total_plays >= 5 && (top_artist_plays as f32 / total_plays as f32) >= 0.40 {
            "The Dedicated Devotee".to_string()
        } else if total_plays >= 10 && (unique_artists as f32 / total_plays as f32) >= 0.60 {
            "The Sonic Explorer".to_string()
        } else if total_minutes >= 5000 {
            "The Heavy Streamer".to_string()
        } else if (9..=17).contains(&fav_hour) {
            "The Deep Focuser".to_string()
        } else {
            "The Harmonic Nomad".to_string()
        }
    }
}

/// Converts day index (0 = Sunday .. 6 = Saturday) to English day name.
pub fn day_index_to_name(day_idx: u8) -> &'static str {
    match day_idx {
        0 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use softify_models::StreamResult;

    fn mock_track(id: &str, title: &str, artist: &str, duration_seconds: u32) -> TrackMetadata {
        TrackMetadata {
            id: id.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            album: None,
            duration_seconds,
            cover_url: None,
        }
    }

    fn mock_stream() -> StreamResult {
        StreamResult {
            stream_url: "https://example.com/stream.mp4".to_string(),
            bitrate: 320000,
            container: "m4a".to_string(),
            codec: "aac".to_string(),
            provider: "jiosaavn_studio_320k".to_string(),
        }
    }

    #[test]
    fn test_compute_wrapped_aggregation() {
        let db = Database::open_in_memory().unwrap();
        let engine = AnalyticsEngine::new(db.clone());

        let t1 = mock_track("t1", "Track 1", "Artist A", 180);
        let t2 = mock_track("t2", "Track 2", "Artist B", 240);
        let stream = mock_stream();

        // Record plays: 3 for t1, 1 for t2
        db.record_playback(&t1, &stream).unwrap();
        db.record_playback(&t1, &stream).unwrap();
        db.record_playback(&t1, &stream).unwrap();
        db.record_playback(&t2, &stream).unwrap();

        let stats = engine.compute_wrapped(None).unwrap();

        // 3*180 + 1*240 = 540 + 240 = 780s = 780,000ms = 13 minutes
        assert_eq!(stats.total_play_time_ms, 780000);
        assert_eq!(stats.total_minutes_listened, 13);

        // Top track: Track 1
        assert_eq!(stats.top_tracks.len(), 2);
        assert_eq!(stats.top_tracks[0].id, "t1");
        assert_eq!(stats.most_played_track.unwrap().id, "t1");

        // Top artist: Artist A
        assert_eq!(stats.top_artists.len(), 2);
        assert_eq!(stats.top_artists[0], "Artist A");
        assert_eq!(stats.unique_artists_count, 2);

        // Listening day & persona present
        assert!(stats.top_listening_day.is_some());
        assert!(stats.audio_persona.is_some());
    }

    #[test]
    fn test_audio_persona_determination() {
        // Night Owl
        assert_eq!(
            AnalyticsEngine::determine_audio_persona(100, 23, 2, 10, 5),
            "The Night Owl"
        );
        assert_eq!(
            AnalyticsEngine::determine_audio_persona(100, 2, 2, 10, 5),
            "The Night Owl"
        );

        // Early Bird
        assert_eq!(
            AnalyticsEngine::determine_audio_persona(100, 6, 2, 10, 5),
            "The Early Bird"
        );

        // Dedicated Devotee (>40% plays on top artist)
        assert_eq!(
            AnalyticsEngine::determine_audio_persona(100, 14, 8, 10, 3),
            "The Dedicated Devotee"
        );

        // Sonic Explorer (>60% unique artist ratio)
        assert_eq!(
            AnalyticsEngine::determine_audio_persona(100, 14, 2, 10, 8),
            "The Sonic Explorer"
        );

        // Heavy Streamer (>= 5000 mins)
        assert_eq!(
            AnalyticsEngine::determine_audio_persona(6000, 14, 3, 20, 5),
            "The Heavy Streamer"
        );

        // Deep Focuser (workday 09:00 - 17:00)
        assert_eq!(
            AnalyticsEngine::determine_audio_persona(100, 11, 2, 10, 4),
            "The Deep Focuser"
        );

        // Empty plays
        assert_eq!(
            AnalyticsEngine::determine_audio_persona(0, 12, 0, 0, 0),
            "The Quiet Observer"
        );
    }

    #[test]
    fn test_day_index_conversion() {
        assert_eq!(day_index_to_name(0), "Sunday");
        assert_eq!(day_index_to_name(5), "Friday");
        assert_eq!(day_index_to_name(6), "Saturday");
        assert_eq!(day_index_to_name(99), "Unknown");
    }
}
