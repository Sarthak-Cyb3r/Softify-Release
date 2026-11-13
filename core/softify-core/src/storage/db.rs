use crate::models::{StreamResult, TrackMetadata};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Opens or creates SQLite database file and initializes tables.
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

    /// Initializes tables and FTS5 full-text search virtual table.
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
            ",
        )?;

        Ok(())
    }

    /// Saves track to cache and indexes it in FTS5.
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
             FROM tracks t
             JOIN tracks_fts fts ON t.id = fts.id
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

    /// Records playback session into playback_history.
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
    }

    #[test]
    fn test_playback_history() {
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

        let history = db.get_playback_history(5).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, "t10");
        assert_eq!(history[0].title, "Song 10");
    }
}
