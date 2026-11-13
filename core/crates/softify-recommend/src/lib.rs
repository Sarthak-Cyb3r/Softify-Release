//! Algorithmic music recommendation and personalized mix generator.
//!
//! Generates:
//! - Daily Mixes 1..6: Distinct artist/subgenre clusters
//! - Weekly Discovery Mix: Fresh unplayed tracks matching user tastes
//! - Mood Mixes: Chill, Workout, Focus, Romance, Party, Melancholy
//! - Strict invariant: Muted tracks and artists are NEVER included

use softify_models::{Mix, MixType, Mood, TrackMetadata};
use softify_storage::{Database, StorageError};
use std::collections::{HashMap, HashSet};

/// Recommendation engine powering Softify personalized mixes.
#[derive(Clone)]
pub struct RecommendEngine {
    db: Database,
}

impl RecommendEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Verifies if a track or its artist is muted ("Never play again").
    pub fn is_muted(&self, track: &TrackMetadata) -> Result<bool, StorageError> {
        if self.db.is_track_muted(&track.id)? {
            return Ok(true);
        }
        if self.db.is_artist_muted(&track.artist)? {
            return Ok(true);
        }
        Ok(false)
    }

    /// Generates Daily Mixes 1 through 6 by clustering favorite and recently played artists.
    pub fn generate_daily_mixes(&self) -> Result<Vec<Mix>, StorageError> {
        // Collect candidate tracks from liked songs and playback history
        let liked = self.db.get_liked_songs(100, 0)?;
        let history = self.db.get_playback_history(100)?;

        let mut candidate_tracks: Vec<TrackMetadata> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        for track in liked.into_iter().chain(history.into_iter()) {
            if seen_ids.insert(track.id.clone()) {
                // STRICT INVARIANT: Always filter out muted tracks or artists
                if !self.is_muted(&track)? {
                    candidate_tracks.push(track);
                }
            }
        }

        if candidate_tracks.is_empty() {
            return Ok(Vec::new());
        }

        // Count artist frequencies to find anchor artists
        let mut artist_counts: HashMap<String, usize> = HashMap::new();
        for track in &candidate_tracks {
            *artist_counts.entry(track.artist.clone()).or_insert(0) += 1;
        }

        let mut sorted_artists: Vec<(String, usize)> = artist_counts.into_iter().collect();
        sorted_artists.sort_by(|a, b| b.1.cmp(&a.1));

        // Define up to 6 distinct daily mix profiles
        let mix_types = [
            (
                MixType::DailyMix1,
                "Daily Mix 1",
                "Pop & High-Energy Anthems",
            ),
            (MixType::DailyMix2, "Daily Mix 2", "Hip-Hop & Urban Rhythms"),
            (
                MixType::DailyMix3,
                "Daily Mix 3",
                "Indie, Alternative & Rock",
            ),
            (
                MixType::DailyMix4,
                "Daily Mix 4",
                "Electronic, Dance & Synth",
            ),
            (
                MixType::DailyMix5,
                "Daily Mix 5",
                "Acoustic & Melodic Harmonies",
            ),
            (MixType::DailyMix6, "Daily Mix 6", "Chillout & Lo-Fi Lounge"),
        ];

        let mut mixes = Vec::new();
        let cluster_count = mix_types.len().min(sorted_artists.len().max(1));

        for (i, (m_type, title, desc_theme)) in mix_types.iter().enumerate().take(cluster_count) {
            // Assign tracks to this mix based on artist hash / round-robin clustering
            let mut mix_tracks = Vec::new();
            let mut featured_artists = Vec::new();

            for track in &candidate_tracks {
                // Cluster assignment via deterministic artist hash
                let artist_hash = track
                    .artist
                    .bytes()
                    .fold(0usize, |acc, b| acc.wrapping_add(b as usize));

                if artist_hash % cluster_count == i || candidate_tracks.len() <= 6 {
                    if !mix_tracks.iter().any(|t: &TrackMetadata| t.id == track.id) {
                        mix_tracks.push(track.clone());
                        if !featured_artists.contains(&track.artist) && featured_artists.len() < 3 {
                            featured_artists.push(track.artist.clone());
                        }
                    }
                }
            }

            if !mix_tracks.is_empty() {
                let description = if featured_artists.is_empty() {
                    desc_theme.to_string()
                } else {
                    format!("Featuring {}, and more", featured_artists.join(", "))
                };

                mixes.push(Mix {
                    id: format!("daily_mix_{}", i + 1),
                    title: title.to_string(),
                    description: Some(description),
                    mix_type: m_type.clone(),
                    tracks: mix_tracks,
                });
            }
        }

        Ok(mixes)
    }

    /// Generates Weekly Discovery Mix with tracks related to top artists, excluding tracks
    /// already played in playback history.
    pub fn generate_weekly_discovery(&self) -> Result<Mix, StorageError> {
        let history = self.db.get_playback_history(200)?;
        let played_ids: HashSet<String> = history.iter().map(|t| t.id.clone()).collect();

        // Get top artists from history and likes
        let liked = self.db.get_liked_songs(50, 0)?;
        let mut artist_counts: HashMap<String, usize> = HashMap::new();
        for t in history.iter().chain(liked.iter()) {
            *artist_counts.entry(t.artist.clone()).or_insert(0) += 1;
        }

        // Search catalog cache for tracks by top artists or cached tracks
        let mut discovery_candidates: Vec<TrackMetadata> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        // Fetch tracks from top artists that user hasn't heard yet
        for (artist, _) in artist_counts.iter().take(10) {
            let artist_tracks = self.db.get_cached_tracks(Some(artist), 20)?;
            for track in artist_tracks {
                if !played_ids.contains(&track.id)
                    && seen_ids.insert(track.id.clone())
                    && !self.is_muted(&track)?
                {
                    discovery_candidates.push(track);
                }
            }
        }

        // Also fetch general catalog tracks not in played_ids
        let general_cached = self.db.get_cached_tracks(None, 50)?;
        for track in general_cached {
            if !played_ids.contains(&track.id)
                && seen_ids.insert(track.id.clone())
                && !self.is_muted(&track)?
            {
                discovery_candidates.push(track);
            }
        }

        Ok(Mix {
            id: "mix_weekly_discovery".to_string(),
            title: "Weekly Discovery".to_string(),
            description: Some(
                "Fresh tracks tailored to your taste that you haven't listened to yet."
                    .to_string(),
            ),
            mix_type: MixType::WeeklyDiscovery,
            tracks: discovery_candidates,
        })
    }

    /// Generates a curated mood mix matching specific mood profiles.
    pub fn generate_mood_mix(&self, mood: Mood) -> Result<Mix, StorageError> {
        // Collect catalog and liked tracks
        let liked = self.db.get_liked_songs(100, 0)?;
        let history = self.db.get_playback_history(100)?;
        let cached = self.db.get_cached_tracks(None, 100)?;

        let (title, description, keywords): (&str, &str, &[&str]) = match mood {
            Mood::Chill => (
                "Chill Mix",
                "Relaxing beats and tranquil vibes to unwind.",
                &[
                    "chill", "relax", "calm", "slow", "ambient", "peace", "soft", "lofi", "quiet",
                ],
            ),
            Mood::Workout => (
                "Workout Mix",
                "High-octane tracks to power your training session.",
                &[
                    "workout", "gym", "run", "pump", "power", "energy", "fast", "heavy", "drive",
                    "beast",
                ],
            ),
            Mood::Focus => (
                "Focus Mix",
                "Instrumental and minimalist soundscapes for deep work.",
                &[
                    "focus",
                    "study",
                    "piano",
                    "instrumental",
                    "deep",
                    "work",
                    "read",
                    "brain",
                ],
            ),
            Mood::Romance => (
                "Romance Mix",
                "Intimate and romantic melodies for special moments.",
                &[
                    "love", "heart", "romance", "kiss", "forever", "sweet", "darling", "baby",
                    "mine",
                ],
            ),
            Mood::Party => (
                "Party Mix",
                "Upbeat bangers and dancefloor essentials.",
                &[
                    "party", "dance", "club", "night", "disco", "rave", "celebrate", "drop",
                    "bass",
                ],
            ),
            Mood::Melancholy => (
                "Melancholy Mix",
                "Reflective and emotive tracks for introspective moods.",
                &[
                    "sad", "blue", "tears", "rain", "hurt", "lonely", "broken", "goodbye",
                    "memory",
                ],
            ),
        };

        let mut matched_tracks = Vec::new();
        let mut seen_ids = HashSet::new();

        for track in liked.into_iter().chain(history).chain(cached) {
            if seen_ids.insert(track.id.clone()) {
                // STRICT INVARIANT: Always filter out muted tracks or artists
                if self.is_muted(&track)? {
                    continue;
                }

                let text = format!(
                    "{} {} {}",
                    track.title.to_lowercase(),
                    track.artist.to_lowercase(),
                    track.album.as_deref().unwrap_or("").to_lowercase()
                );

                if keywords.iter().any(|&kw| text.contains(kw)) {
                    matched_tracks.push(track);
                }
            }
        }

        Ok(Mix {
            id: format!("mix_mood_{:?}", mood).to_lowercase(),
            title: title.to_string(),
            description: Some(description.to_string()),
            mix_type: MixType::mood(mood),
            tracks: matched_tracks,
        })
    }

    /// Generates all mood mixes (Chill, Workout, Focus, Romance, Party, Melancholy).
    pub fn generate_all_mood_mixes(&self) -> Result<Vec<Mix>, StorageError> {
        let moods = [
            Mood::Chill,
            Mood::Workout,
            Mood::Focus,
            Mood::Romance,
            Mood::Party,
            Mood::Melancholy,
        ];

        let mut mixes = Vec::new();
        for mood in moods {
            mixes.push(self.generate_mood_mix(mood)?);
        }
        Ok(mixes)
    }

    /// Generates full suite of mixes (Daily mixes 1..6, Weekly Discovery, and Mood mixes).
    pub fn generate_all_mixes(&self) -> Result<Vec<Mix>, StorageError> {
        let mut all = self.generate_daily_mixes()?;
        all.push(self.generate_weekly_discovery()?);
        all.extend(self.generate_all_mood_mixes()?);
        Ok(all)
    }

    /// Asynchronously generates Daily Mixes 1..6 on a background thread.
    pub async fn generate_daily_mixes_async(&self) -> Result<Vec<Mix>, StorageError> {
        let engine = self.clone();
        tokio::task::spawn_blocking(move || engine.generate_daily_mixes())
            .await
            .map_err(|e| softify_storage::StorageError::ToSqlConversionFailure(Box::new(e)))?
    }

    /// Asynchronously generates Weekly Discovery Mix on a background thread.
    pub async fn generate_weekly_discovery_async(&self) -> Result<Mix, StorageError> {
        let engine = self.clone();
        tokio::task::spawn_blocking(move || engine.generate_weekly_discovery())
            .await
            .map_err(|e| softify_storage::StorageError::ToSqlConversionFailure(Box::new(e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use softify_models::StreamResult;

    fn mock_track(id: &str, title: &str, artist: &str) -> TrackMetadata {
        TrackMetadata {
            id: id.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            album: None,
            duration_seconds: 200,
            cover_url: None,
        }
    }

    fn mock_stream(url: &str) -> StreamResult {
        StreamResult {
            stream_url: url.to_string(),
            bitrate: 320000,
            container: "m4a".to_string(),
            codec: "aac".to_string(),
            provider: "jiosaavn_studio_320k".to_string(),
        }
    }

    #[test]
    fn test_daily_mix_generation_and_clustering() {
        let db = Database::open_in_memory().unwrap();
        let engine = RecommendEngine::new(db.clone());

        let t1 = mock_track("t1", "Blinding Lights", "The Weeknd");
        let t2 = mock_track("t2", "One More Time", "Daft Punk");
        let t3 = mock_track("t3", "Starboy", "The Weeknd");

        db.like_song(&t1).unwrap();
        db.like_song(&t2).unwrap();
        db.record_playback(&t3, &mock_stream("https://example.com/3")).unwrap();

        let daily_mixes = engine.generate_daily_mixes().unwrap();
        assert!(!daily_mixes.is_empty());

        for mix in &daily_mixes {
            assert!(!mix.tracks.is_empty());
        }
    }

    #[test]
    fn test_strict_invariant_muted_tracks_and_artists_never_included() {
        let db = Database::open_in_memory().unwrap();
        let engine = RecommendEngine::new(db.clone());

        let good_track = mock_track("good_1", "Levitating", "Dua Lipa");
        let muted_track = mock_track("bad_1", "Annoying Track", "Good Artist");
        let muted_artist_track = mock_track("bad_2", "Any Song", "Muted Artist");

        db.like_song(&good_track).unwrap();
        db.like_song(&muted_track).unwrap();
        db.like_song(&muted_artist_track).unwrap();

        // Mute the specific track bad_1
        db.mute_track(&muted_track).unwrap();
        // Mute the artist "Muted Artist"
        db.mute_track(&muted_artist_track).unwrap();

        // Verify Daily Mixes never contain muted track or artist
        let daily = engine.generate_daily_mixes().unwrap();
        for mix in daily {
            for track in mix.tracks {
                assert_ne!(track.id, "bad_1");
                assert_ne!(track.artist, "Muted Artist");
            }
        }

        // Verify Weekly Discovery never contains muted track or artist
        let discovery = engine.generate_weekly_discovery().unwrap();
        for track in discovery.tracks {
            assert_ne!(track.id, "bad_1");
            assert_ne!(track.artist, "Muted Artist");
        }

        // Verify Mood Mixes never contain muted track or artist
        let mood = engine.generate_mood_mix(Mood::Party).unwrap();
        for track in mood.tracks {
            assert_ne!(track.id, "bad_1");
            assert_ne!(track.artist, "Muted Artist");
        }
    }

    #[test]
    fn test_weekly_discovery_excludes_playback_history() {
        let db = Database::open_in_memory().unwrap();
        let engine = RecommendEngine::new(db.clone());

        let played_track = mock_track("played_1", "Already Played Song", "Favorite Artist");
        let fresh_track = mock_track("fresh_1", "Unheard Gem", "Favorite Artist");

        // Cache both tracks in catalog
        db.save_track(&played_track, "catalog").unwrap();
        db.save_track(&fresh_track, "catalog").unwrap();

        // Record playback for played_track
        db.record_playback(&played_track, &mock_stream("https://example.com/stream")).unwrap();

        let discovery = engine.generate_weekly_discovery().unwrap();
        assert_eq!(discovery.mix_type, MixType::WeeklyDiscovery);

        // Discovery MUST include fresh_track and MUST exclude played_track
        let ids: Vec<String> = discovery.tracks.into_iter().map(|t| t.id).collect();
        assert!(ids.contains(&"fresh_1".to_string()));
        assert!(!ids.contains(&"played_1".to_string()));
    }

    #[test]
    fn test_mood_mix_categorization() {
        let db = Database::open_in_memory().unwrap();
        let engine = RecommendEngine::new(db.clone());

        let workout_track = mock_track("w1", "Heavy Workout Pump", "Fitness Band");
        let chill_track = mock_track("c1", "Calm Relax Ambient Breeze", "Chillout Artist");

        db.save_track(&workout_track, "catalog").unwrap();
        db.save_track(&chill_track, "catalog").unwrap();

        let workout_mix = engine.generate_mood_mix(Mood::Workout).unwrap();
        assert_eq!(workout_mix.mix_type, MixType::mood(Mood::Workout));
        assert!(workout_mix.tracks.iter().any(|t| t.id == "w1"));
        assert!(!workout_mix.tracks.iter().any(|t| t.id == "c1"));

        let chill_mix = engine.generate_mood_mix(Mood::Chill).unwrap();
        assert_eq!(chill_mix.mix_type, MixType::mood(Mood::Chill));
        assert!(chill_mix.tracks.iter().any(|t| t.id == "c1"));
        assert!(!chill_mix.tracks.iter().any(|t| t.id == "w1"));
    }
}
