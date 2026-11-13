use rusqlite::{params, Connection};
use softify_models::{Playlist, StreamResult, TrackMetadata, WrappedStats};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_id(prefix: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let count = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}_{now:x}_{count:x}")
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Opens or creates SQLite database file and initializes all tables and indexes.
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    /// Opens an in-memory SQLite database for testing and ephemeral caching.
    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    /// Initializes tables, FTS5 virtual tables, and indexes according to specification.
    fn init_schema(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT,
                duration_seconds INTEGER NOT NULL,
                cover_url TEXT,
                source TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS tracks_fts USING fts5(
                id UNINDEXED,
                title,
                artist,
                album
            );

            CREATE TABLE IF NOT EXISTS playback_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                track_id TEXT NOT NULL,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT,
                duration_seconds INTEGER NOT NULL,
                cover_url TEXT,
                stream_url TEXT NOT NULL,
                bitrate INTEGER NOT NULL,
                provider TEXT NOT NULL,
                played_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS liked_songs (
                track_id TEXT PRIMARY KEY,
                added_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                cover_url TEXT,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT NOT NULL,
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                added_at INTEGER NOT NULL,
                PRIMARY KEY (playlist_id, track_id)
            );

            CREATE TABLE IF NOT EXISTS muted_tracks (
                track_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                muted_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_playback_history_track_id ON playback_history(track_id);
            CREATE INDEX IF NOT EXISTS idx_playback_history_played_at ON playback_history(played_at);
            CREATE INDEX IF NOT EXISTS idx_liked_songs_added_at ON liked_songs(added_at);
            CREATE INDEX IF NOT EXISTS idx_playlist_tracks_position ON playlist_tracks(playlist_id, position);
            CREATE INDEX IF NOT EXISTS idx_muted_tracks_artist ON muted_tracks(artist);
            ",
        )?;

        Ok(())
    }

    // ==========================================
    // CATALOG CACHE & FTS5 FULL-TEXT SEARCH
    // ==========================================

    /// Saves track to cache and indexes it into FTS5.
    pub fn save_track(&self, track: &TrackMetadata, source: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO tracks (id, title, artist, album, duration_seconds, cover_url, source, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                track.id,
                track.title,
                track.artist,
                track.album,
                track.duration_seconds,
                track.cover_url,
                source,
                now
            ],
        )?;

        // Update FTS5 virtual table
        conn.execute(
            "DELETE FROM tracks_fts WHERE id = ?1",
            params![track.id],
        )?;
        conn.execute(
            "INSERT INTO tracks_fts (id, title, artist, album) VALUES (?1, ?2, ?3, ?4)",
            params![track.id, track.title, track.artist, track.album],
        )?;

        Ok(())
    }

    /// Retrieves a track by ID from cache.
    pub fn get_track(&self, id: &str) -> Result<Option<TrackMetadata>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, album, duration_seconds, cover_url FROM tracks WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(TrackMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_seconds: row.get(4)?,
                cover_url: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Performs full-text search against cached catalog via SQLite FTS5.
    pub fn search_tracks_fts(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<TrackMetadata>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();

        // Sanitize FTS5 query to prevent syntax errors
        let sanitized = query
            .split_whitespace()
            .map(|word| format!("\"{}\"*", word.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" ");

        if sanitized.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = conn.prepare(
            "SELECT t.id, t.title, t.artist, t.album, t.duration_seconds, t.cover_url
             FROM tracks_fts fts
             JOIN tracks t ON fts.id = t.id
             WHERE tracks_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![sanitized, limit as i64], |row| {
            Ok(TrackMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_seconds: row.get(4)?,
                cover_url: row.get(5)?,
            })
        })?;

        let mut tracks = Vec::new();
        for track in rows {
            tracks.push(track?);
        }

        Ok(tracks)
    }

    /// Returns cached tracks matching an optional artist name, or all cached tracks if artist is None.
    pub fn get_cached_tracks(
        &self,
        artist: Option<&str>,
        limit: usize,
    ) -> Result<Vec<TrackMetadata>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut tracks = Vec::new();

        if let Some(art) = artist {
            let mut stmt = conn.prepare(
                "SELECT id, title, artist, album, duration_seconds, cover_url
                 FROM tracks
                 WHERE artist LIKE ?1
                 ORDER BY cached_at DESC
                 LIMIT ?2",
            )?;
            let pattern = format!("%{art}%");
            let rows = stmt.query_map(params![pattern, limit as i64], |row| {
                Ok(TrackMetadata {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    album: row.get(3)?,
                    duration_seconds: row.get(4)?,
                    cover_url: row.get(5)?,
                })
            })?;
            for t in rows {
                tracks.push(t?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, title, artist, album, duration_seconds, cover_url
                 FROM tracks
                 ORDER BY cached_at DESC
                 LIMIT ?1",
            )?;
            let rows = stmt.query_map(params![limit as i64], |row| {
                Ok(TrackMetadata {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    album: row.get(3)?,
                    duration_seconds: row.get(4)?,
                    cover_url: row.get(5)?,
                })
            })?;
            for t in rows {
                tracks.push(t?);
            }
        }

        Ok(tracks)
    }

    /// Deletes a track from cache and FTS5.
    pub fn delete_track(&self, id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
        conn.execute("DELETE FROM tracks_fts WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==========================================
    // PLAYBACK HISTORY & ANALYTICS
    // ==========================================

    /// Records playback session into playback_history and ensures track is in tracks table.
    pub fn record_playback(
        &self,
        track: &TrackMetadata,
        stream: &StreamResult,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Ensure track is in tracks table
        conn.execute(
            "INSERT OR IGNORE INTO tracks (id, title, artist, album, duration_seconds, cover_url, source, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                track.id,
                track.title,
                track.artist,
                track.album,
                track.duration_seconds,
                track.cover_url,
                "history",
                now
            ],
        )?;

        conn.execute(
            "INSERT INTO playback_history (track_id, title, artist, album, duration_seconds, cover_url, stream_url, bitrate, provider, played_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                track.id,
                track.title,
                track.artist,
                track.album,
                track.duration_seconds,
                track.cover_url,
                stream.stream_url,
                stream.bitrate,
                stream.provider,
                now
            ],
        )?;

        Ok(())
    }

    /// Retrieves recently played tracks from playback history.
    pub fn get_playback_history(&self, limit: usize) -> Result<Vec<TrackMetadata>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT track_id, title, artist, album, duration_seconds, cover_url
             FROM playback_history
             ORDER BY played_at DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(TrackMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_seconds: row.get(4)?,
                cover_url: row.get(5)?,
            })
        })?;

        let mut history = Vec::new();
        for item in rows {
            history.push(item?);
        }

        Ok(history)
    }

    /// Clears all entries from playback history.
    pub fn clear_playback_history(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM playback_history", [])?;
        Ok(())
    }

    /// Calculates listening summary and WrappedStats from playback history.
    pub fn get_wrapped_stats(&self, since_epoch: Option<i64>) -> Result<WrappedStats, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let since = since_epoch.unwrap_or(0);

        // 1. Total play time in ms
        let mut total_time_stmt = conn.prepare(
            "SELECT COALESCE(SUM(duration_seconds), 0) FROM playback_history WHERE played_at >= ?1",
        )?;
        let total_seconds: i64 = total_time_stmt.query_row(params![since], |r| r.get(0))?;
        let total_play_time_ms = (total_seconds.max(0) as u64) * 1000;

        // 2. Top tracks
        let mut top_tracks_stmt = conn.prepare(
            "SELECT track_id, title, artist, album, duration_seconds, cover_url, COUNT(*) as play_count
             FROM playback_history
             WHERE played_at >= ?1
             GROUP BY track_id
             ORDER BY play_count DESC
             LIMIT 10",
        )?;
        let top_tracks_rows = top_tracks_stmt.query_map(params![since], |row| {
            Ok(TrackMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_seconds: row.get(4)?,
                cover_url: row.get(5)?,
            })
        })?;
        let mut top_tracks = Vec::new();
        for track in top_tracks_rows {
            top_tracks.push(track?);
        }

        let most_played_track = top_tracks.first().cloned();

        // 3. Top artists
        let mut top_artists_stmt = conn.prepare(
            "SELECT artist, COUNT(*) as play_count
             FROM playback_history
             WHERE played_at >= ?1
             GROUP BY artist
             ORDER BY play_count DESC
             LIMIT 5",
        )?;
        let top_artists_rows = top_artists_stmt.query_map(params![since], |row| row.get::<_, String>(0))?;
        let mut top_artists = Vec::new();
        for artist in top_artists_rows {
            top_artists.push(artist?);
        }

        // 4. Unique artists count
        let mut unique_artists_stmt = conn.prepare(
            "SELECT COUNT(DISTINCT artist) FROM playback_history WHERE played_at >= ?1",
        )?;
        let unique_artists_count: u32 = unique_artists_stmt.query_row(params![since], |r| r.get(0))?;

        // 5. Favorite listening hour (0..23)
        let mut hour_stmt = conn.prepare(
            "SELECT CAST(strftime('%H', played_at, 'unixepoch') AS INTEGER) as hour, COUNT(*) as count
             FROM playback_history
             WHERE played_at >= ?1
             GROUP BY hour
             ORDER BY count DESC
             LIMIT 1",
        )?;
        let favorite_listening_hour: u8 = hour_stmt
            .query_row(params![since], |r| r.get::<_, u8>(0))
            .unwrap_or(12);

        // 6. Top listening day of the week
        let mut day_stmt = conn.prepare(
            "SELECT CAST(strftime('%w', played_at, 'unixepoch') AS INTEGER) as day_of_week, COUNT(*) as count
             FROM playback_history
             WHERE played_at >= ?1
             GROUP BY day_of_week
             ORDER BY count DESC
             LIMIT 1",
        )?;
        let top_day_idx: Option<u8> = day_stmt.query_row(params![since], |r| r.get(0)).ok();
        let top_listening_day = top_day_idx.map(|idx| match idx {
            0 => "Sunday",
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            _ => "Everyday",
        }.to_string());

        let total_minutes_listened = total_play_time_ms / 60_000;

        let audio_persona = if total_seconds == 0 {
            None
        } else if favorite_listening_hour >= 22 || favorite_listening_hour <= 4 {
            Some("The Night Owl".to_string())
        } else if (5..=8).contains(&favorite_listening_hour) {
            Some("The Early Bird".to_string())
        } else if top_artists.len() == 1 || (top_tracks.len() > 0 && unique_artists_count == 1) {
            Some("The Dedicated Devotee".to_string())
        } else if unique_artists_count > 10 {
            Some("The Sonic Explorer".to_string())
        } else if total_seconds >= 300_000 {
            Some("The Heavy Streamer".to_string())
        } else {
            Some("The Harmonic Nomad".to_string())
        };

        Ok(WrappedStats {
            total_play_time_ms,
            total_minutes_listened,
            top_tracks,
            top_artists,
            most_played_track,
            unique_artists_count,
            favorite_listening_hour,
            top_listening_day,
            audio_persona,
        })
    }

    // ==========================================
    // LIKED SONGS
    // ==========================================

    /// Likes a song and ensures it exists in cached tracks.
    pub fn like_song(&self, track: &TrackMetadata) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Ensure track is cached in tracks table
        conn.execute(
            "INSERT OR IGNORE INTO tracks (id, title, artist, album, duration_seconds, cover_url, source, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                track.id,
                track.title,
                track.artist,
                track.album,
                track.duration_seconds,
                track.cover_url,
                "liked",
                now
            ],
        )?;

        conn.execute(
            "INSERT OR REPLACE INTO liked_songs (track_id, added_at) VALUES (?1, ?2)",
            params![track.id, now],
        )?;

        Ok(())
    }

    /// Unlikes a song by track ID.
    pub fn unlike_song(&self, track_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM liked_songs WHERE track_id = ?1",
            params![track_id],
        )?;
        Ok(())
    }

    /// Checks if a song is marked as liked.
    pub fn is_song_liked(&self, track_id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT 1 FROM liked_songs WHERE track_id = ?1")?;
        let mut rows = stmt.query(params![track_id])?;
        Ok(rows.next()?.is_some())
    }

    /// Retrieves liked songs ordered by newest liked first.
    pub fn get_liked_songs(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<TrackMetadata>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.title, t.artist, t.album, t.duration_seconds, t.cover_url
             FROM liked_songs l
             JOIN tracks t ON l.track_id = t.id
             ORDER BY l.added_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            Ok(TrackMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_seconds: row.get(4)?,
                cover_url: row.get(5)?,
            })
        })?;

        let mut songs = Vec::new();
        for song in rows {
            songs.push(song?);
        }

        Ok(songs)
    }

    /// Returns the total count of liked songs.
    pub fn get_liked_songs_count(&self) -> Result<u32, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM liked_songs")?;
        let count: u32 = stmt.query_row([], |r| r.get(0))?;
        Ok(count)
    }

    // ==========================================
    // PLAYLIST MANAGEMENT
    // ==========================================

    /// Creates a new playlist with auto-generated unique ID.
    pub fn create_playlist(
        &self,
        title: &str,
        description: Option<&str>,
        cover_url: Option<&str>,
    ) -> Result<Playlist, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let id = generate_id("pl");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO playlists (id, title, description, cover_url, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, title, description, cover_url, now],
        )?;

        Ok(Playlist {
            id,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            cover_url: cover_url.map(|s| s.to_string()),
            track_count: 0,
            created_at: now,
        })
    }

    /// Retrieves playlist details and track count by ID.
    pub fn get_playlist(&self, id: &str) -> Result<Option<Playlist>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.title, p.description, p.cover_url, p.created_at, COUNT(pt.track_id)
             FROM playlists p
             LEFT JOIN playlist_tracks pt ON p.id = pt.playlist_id
             WHERE p.id = ?1
             GROUP BY p.id",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Playlist {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                cover_url: row.get(3)?,
                created_at: row.get(4)?,
                track_count: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Lists all playlists with their track counts.
    pub fn list_playlists(&self) -> Result<Vec<Playlist>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.title, p.description, p.cover_url, p.created_at, COUNT(pt.track_id)
             FROM playlists p
             LEFT JOIN playlist_tracks pt ON p.id = pt.playlist_id
             GROUP BY p.id
             ORDER BY p.created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                cover_url: row.get(3)?,
                created_at: row.get(4)?,
                track_count: row.get(5)?,
            })
        })?;

        let mut playlists = Vec::new();
        for p in rows {
            playlists.push(p?);
        }

        Ok(playlists)
    }

    /// Updates playlist metadata.
    pub fn update_playlist(
        &self,
        id: &str,
        title: &str,
        description: Option<&str>,
        cover_url: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE playlists SET title = ?1, description = ?2, cover_url = ?3 WHERE id = ?4",
            params![title, description, cover_url, id],
        )?;
        Ok(())
    }

    /// Deletes a playlist and its track associations.
    pub fn delete_playlist(&self, id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM playlist_tracks WHERE playlist_id = ?1", params![id])?;
        conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Adds a track to the end of a playlist.
    pub fn add_track_to_playlist(
        &self,
        playlist_id: &str,
        track: &TrackMetadata,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Ensure track is in tracks table
        conn.execute(
            "INSERT OR IGNORE INTO tracks (id, title, artist, album, duration_seconds, cover_url, source, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                track.id,
                track.title,
                track.artist,
                track.album,
                track.duration_seconds,
                track.cover_url,
                "playlist",
                now
            ],
        )?;

        // Find max position
        let mut pos_stmt = conn.prepare(
            "SELECT COALESCE(MAX(position), -1) FROM playlist_tracks WHERE playlist_id = ?1",
        )?;
        let max_pos: i32 = pos_stmt.query_row(params![playlist_id], |r| r.get(0))?;
        let next_pos = max_pos + 1;

        conn.execute(
            "INSERT OR REPLACE INTO playlist_tracks (playlist_id, track_id, position, added_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![playlist_id, track.id, next_pos, now],
        )?;

        Ok(())
    }

    /// Removes a track from a playlist.
    pub fn remove_track_from_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
            params![playlist_id, track_id],
        )?;
        Ok(())
    }

    /// Retrieves ordered list of tracks in a playlist.
    pub fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<TrackMetadata>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.title, t.artist, t.album, t.duration_seconds, t.cover_url
             FROM playlist_tracks pt
             JOIN tracks t ON pt.track_id = t.id
             WHERE pt.playlist_id = ?1
             ORDER BY pt.position ASC",
        )?;

        let rows = stmt.query_map(params![playlist_id], |row| {
            Ok(TrackMetadata {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_seconds: row.get(4)?,
                cover_url: row.get(5)?,
            })
        })?;

        let mut tracks = Vec::new();
        for track in rows {
            tracks.push(track?);
        }

        Ok(tracks)
    }

    /// Reorders tracks in a playlist to match the given track_ids slice order.
    pub fn reorder_playlist_tracks(
        &self,
        playlist_id: &str,
        track_ids: &[String],
    ) -> Result<(), rusqlite::Error> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        for (pos, track_id) in track_ids.iter().enumerate() {
            tx.execute(
                "UPDATE playlist_tracks SET position = ?1 WHERE playlist_id = ?2 AND track_id = ?3",
                params![pos as i32, playlist_id, track_id],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    // ==========================================
    // MUTED TRACKS ("NEVER PLAY AGAIN")
    // ==========================================

    /// Mutes a track so it will never be recommended or played again.
    pub fn mute_track(&self, track: &TrackMetadata) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO muted_tracks (track_id, title, artist, muted_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![track.id, track.title, track.artist, now],
        )?;

        Ok(())
    }

    /// Unmutes a track by track ID.
    pub fn unmute_track(&self, track_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM muted_tracks WHERE track_id = ?1",
            params![track_id],
        )?;
        Ok(())
    }

    /// Checks if a track ID is muted.
    pub fn is_track_muted(&self, track_id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT 1 FROM muted_tracks WHERE track_id = ?1")?;
        let mut rows = stmt.query(params![track_id])?;
        Ok(rows.next()?.is_some())
    }

    /// Checks if any muted track matches or contains the given artist name.
    pub fn is_artist_muted(&self, artist: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let lower = artist.to_lowercase();
        let mut stmt = conn.prepare(
            "SELECT 1 FROM muted_tracks
             WHERE LOWER(artist) = ?1 OR ?1 LIKE '%' || LOWER(artist) || '%' OR LOWER(artist) LIKE '%' || ?1 || '%'
             LIMIT 1",
        )?;
        let mut rows = stmt.query(params![lower])?;
        Ok(rows.next()?.is_some())
    }

    /// Returns list of all muted tracks as (track_id, title, artist).
    pub fn get_muted_tracks(&self) -> Result<Vec<(String, String, String)>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT track_id, title, artist FROM muted_tracks ORDER BY muted_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut muted = Vec::new();
        for item in rows {
            muted.push(item?);
        }

        Ok(muted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_fts5_indexing_and_search() {
        let db = Database::open_in_memory().expect("In-memory DB creation failed");

        let track1 = TrackMetadata {
            id: "t1".to_string(),
            title: "Blinding Lights".to_string(),
            artist: "The Weeknd".to_string(),
            album: Some("After Hours".to_string()),
            duration_seconds: 200,
            cover_url: Some("https://example.com/cover1.jpg".to_string()),
        };

        let track2 = TrackMetadata {
            id: "t2".to_string(),
            title: "Starboy".to_string(),
            artist: "The Weeknd, Daft Punk".to_string(),
            album: Some("Starboy".to_string()),
            duration_seconds: 230,
            cover_url: None,
        };

        db.save_track(&track1, "spotify").unwrap();
        db.save_track(&track2, "spotify").unwrap();

        // Exact ID lookup
        let fetched = db.get_track("t1").unwrap().expect("Track should exist");
        assert_eq!(fetched.title, "Blinding Lights");

        // FTS5 Search by title
        let search_title = db.search_tracks_fts("Blinding", 10).unwrap();
        assert_eq!(search_title.len(), 1);
        assert_eq!(search_title[0].id, "t1");

        // FTS5 Search by artist
        let search_artist = db.search_tracks_fts("Weeknd", 10).unwrap();
        assert_eq!(search_artist.len(), 2);

        // FTS5 Search by album
        let search_album = db.search_tracks_fts("Hours", 10).unwrap();
        assert_eq!(search_album.len(), 1);
        assert_eq!(search_album[0].title, "Blinding Lights");

        // Delete track
        db.delete_track("t1").unwrap();
        assert!(db.get_track("t1").unwrap().is_none());
        assert_eq!(db.search_tracks_fts("Blinding", 10).unwrap().len(), 0);
    }

    #[test]
    fn test_playback_history_and_wrapped_stats() {
        let db = Database::open_in_memory().expect("DB init failed");

        let track = TrackMetadata {
            id: "t10".to_string(),
            title: "Song 10".to_string(),
            artist: "Artist 10".to_string(),
            album: None,
            duration_seconds: 180,
            cover_url: None,
        };

        let stream = StreamResult {
            stream_url: "https://example.com/stream_320.mp4".to_string(),
            bitrate: 320000,
            container: "m4a".to_string(),
            codec: "aac".to_string(),
            provider: "jiosaavn_studio_320k".to_string(),
        };

        db.record_playback(&track, &stream).unwrap();
        db.record_playback(&track, &stream).unwrap();

        let history = db.get_playback_history(5).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].id, "t10");

        let stats = db.get_wrapped_stats(None).unwrap();
        assert_eq!(stats.total_play_time_ms, 360000);
        assert_eq!(stats.total_minutes_listened, 6);
        assert_eq!(stats.top_tracks.len(), 1);
        assert_eq!(stats.top_artists.len(), 1);
        assert_eq!(stats.unique_artists_count, 1);
        assert_eq!(stats.most_played_track.unwrap().id, "t10");
        assert!(stats.top_listening_day.is_some());
        assert!(stats.audio_persona.is_some());

        db.clear_playback_history().unwrap();
        assert_eq!(db.get_playback_history(5).unwrap().len(), 0);
    }

    #[test]
    fn test_liked_songs_crud() {
        let db = Database::open_in_memory().expect("DB init failed");

        let track = TrackMetadata {
            id: "liked_1".to_string(),
            title: "Favorite Song".to_string(),
            artist: "Favorite Artist".to_string(),
            album: Some("Favorite Album".to_string()),
            duration_seconds: 210,
            cover_url: None,
        };

        assert!(!db.is_song_liked("liked_1").unwrap());
        db.like_song(&track).unwrap();
        assert!(db.is_song_liked("liked_1").unwrap());
        assert_eq!(db.get_liked_songs_count().unwrap(), 1);

        let liked = db.get_liked_songs(10, 0).unwrap();
        assert_eq!(liked.len(), 1);
        assert_eq!(liked[0].id, "liked_1");

        db.unlike_song("liked_1").unwrap();
        assert!(!db.is_song_liked("liked_1").unwrap());
        assert_eq!(db.get_liked_songs_count().unwrap(), 0);
    }

    #[test]
    fn test_playlist_lifecycle_and_reordering() {
        let db = Database::open_in_memory().expect("DB init failed");

        let playlist = db
            .create_playlist("Chill Vibes", Some("Evening relaxation"), None)
            .unwrap();
        assert_eq!(playlist.title, "Chill Vibes");
        assert_eq!(playlist.track_count, 0);

        let track1 = TrackMetadata {
            id: "pl_t1".to_string(),
            title: "Track 1".to_string(),
            artist: "Artist 1".to_string(),
            album: None,
            duration_seconds: 180,
            cover_url: None,
        };
        let track2 = TrackMetadata {
            id: "pl_t2".to_string(),
            title: "Track 2".to_string(),
            artist: "Artist 2".to_string(),
            album: None,
            duration_seconds: 200,
            cover_url: None,
        };

        db.add_track_to_playlist(&playlist.id, &track1).unwrap();
        db.add_track_to_playlist(&playlist.id, &track2).unwrap();

        let tracks = db.get_playlist_tracks(&playlist.id).unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].id, "pl_t1");
        assert_eq!(tracks[1].id, "pl_t2");

        let fetched = db.get_playlist(&playlist.id).unwrap().unwrap();
        assert_eq!(fetched.track_count, 2);

        // Reorder tracks
        db.reorder_playlist_tracks(
            &playlist.id,
            &["pl_t2".to_string(), "pl_t1".to_string()],
        )
        .unwrap();

        let reordered = db.get_playlist_tracks(&playlist.id).unwrap();
        assert_eq!(reordered[0].id, "pl_t2");
        assert_eq!(reordered[1].id, "pl_t1");

        // Remove track
        db.remove_track_from_playlist(&playlist.id, "pl_t2").unwrap();
        assert_eq!(db.get_playlist_tracks(&playlist.id).unwrap().len(), 1);

        // Update playlist
        db.update_playlist(
            &playlist.id,
            "Updated Vibes",
            Some("New description"),
            Some("https://example.com/pic.jpg"),
        )
        .unwrap();
        let updated = db.get_playlist(&playlist.id).unwrap().unwrap();
        assert_eq!(updated.title, "Updated Vibes");
        assert_eq!(updated.description.as_deref(), Some("New description"));

        // Delete playlist
        db.delete_playlist(&playlist.id).unwrap();
        assert!(db.get_playlist(&playlist.id).unwrap().is_none());
    }

    #[test]
    fn test_muted_tracks() {
        let db = Database::open_in_memory().expect("DB init failed");

        let track = TrackMetadata {
            id: "mute_1".to_string(),
            title: "Annoying Song".to_string(),
            artist: "Bad Singer".to_string(),
            album: None,
            duration_seconds: 150,
            cover_url: None,
        };

        assert!(!db.is_track_muted("mute_1").unwrap());
        assert!(!db.is_artist_muted("Bad Singer").unwrap());

        db.mute_track(&track).unwrap();

        assert!(db.is_track_muted("mute_1").unwrap());
        assert!(db.is_artist_muted("Bad Singer").unwrap());
        assert!(db.is_artist_muted("bad singer").unwrap());
        assert!(db.is_artist_muted("Bad Singer feat. Guest").unwrap());

        let muted_list = db.get_muted_tracks().unwrap();
        assert_eq!(muted_list.len(), 1);
        assert_eq!(muted_list[0].0, "mute_1");

        db.unmute_track("mute_1").unwrap();
        assert!(!db.is_track_muted("mute_1").unwrap());
    }
}
