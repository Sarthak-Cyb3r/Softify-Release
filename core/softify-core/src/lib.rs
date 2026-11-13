pub mod crypto;
pub mod downloader;
pub mod models;
pub mod normalizer;
pub mod resolvers;
pub mod storage;

pub use models::{AudioQuality, SearchResult, StreamResult, TrackMetadata};

uniffi::setup_scaffolding!();

use crate::downloader::{download_audio_stream, get_track_filename, is_offline_ready};
use crate::resolvers::hybrid::HybridResolver;
use crate::resolvers::jiosaavn::JioSaavnResolver;
use crate::resolvers::lyrics::LyricsResolver;
use crate::resolvers::spotify::SpotifyResolver;
use crate::storage::db::Database;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Structured error variants exposed across UniFFI to Kotlin as typed exceptions.
#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum SoftifyError {
    #[error("Initialization error: {msg}")]
    InitError { msg: String },
    #[error("Search error: {msg}")]
    SearchError { msg: String },
    #[error("Resolution error: {msg}")]
    ResolutionError { msg: String },
    #[error("Database error: {msg}")]
    DatabaseError { msg: String },
    #[error("Crypto error: {msg}")]
    CryptoError { msg: String },
    #[error("Network error: {msg}")]
    NetworkError { msg: String },
    #[error("Download error: {msg}")]
    DownloadError { msg: String },
}

/// Service trait exposed to Kotlin / Multiplatform callers via UniFFI.
pub trait SoftifyEngine: Send + Sync {
    fn search(&self, query: String) -> Vec<TrackMetadata>;
    fn resolve_stream(&self, track: TrackMetadata) -> Result<StreamResult, String>;
    fn get_lyrics(&self, title: String, artist: String) -> Option<String>;
}

/// Primary Rust Core Engine instance exposed via UniFFI.
#[derive(uniffi::Object)]
pub struct SoftifyEngineService {
    runtime: tokio::runtime::Runtime,
    spotify: SpotifyResolver,
    jiosaavn: JioSaavnResolver,
    hybrid: HybridResolver,
    lyrics: LyricsResolver,
    db: Database,
    http_client: reqwest::Client,
}

#[uniffi::export]
impl SoftifyEngineService {
    /// Creates a new SoftifyEngineService instance.
    /// If db_path is None, an in-memory SQLite database is used.
    #[uniffi::constructor]
    pub fn new(db_path: Option<String>) -> Result<Arc<Self>, SoftifyError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .map_err(|e| SoftifyError::InitError {
                msg: format!("Failed to initialize Tokio runtime: {e}"),
            })?;

        let db = match db_path {
            Some(path) => Database::open(&path).map_err(|e| SoftifyError::DatabaseError {
                msg: format!("Failed to open SQLite database at '{path}': {e}"),
            })?,
            None => Database::open_in_memory().map_err(|e| SoftifyError::DatabaseError {
                msg: format!("Failed to open in-memory SQLite database: {e}"),
            })?,
        };

        let http_client = reqwest::Client::builder()
            .user_agent("Softify-NextGen/1.0 (Linux; Android)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Ok(Arc::new(Self {
            runtime,
            spotify: SpotifyResolver::new(),
            jiosaavn: JioSaavnResolver::new(),
            hybrid: HybridResolver::new(),
            lyrics: LyricsResolver::new(),
            db,
            http_client,
        }))
    }

    /// Searches Spotify Pathfinder (primary) and JioSaavn (fallback), caching results in SQLite FTS5.
    pub fn search(&self, query: String) -> Vec<TrackMetadata> {
        let spotify = self.spotify.clone();
        let jiosaavn = self.jiosaavn.clone();
        let q = query.clone();

        let tracks = self.runtime.block_on(async move {
            // Primary metadata search: Spotify Pathfinder
            match spotify.search(&q, 20).await {
                Ok(tracks) if !tracks.is_empty() => tracks,
                _ => {
                    // Fallback search: JioSaavn
                    jiosaavn.search(&q).await.unwrap_or_default()
                }
            }
        });

        // Cache results in local SQLite FTS5 database
        for track in &tracks {
            let _ = self.db.save_track(track, "catalog");
        }

        tracks
    }

    /// Resolves stream audio URL guaranteeing 320 Kbps CD-quality for JioSaavn or YTM fallback.
    pub fn resolve_stream(&self, track: TrackMetadata) -> Result<StreamResult, SoftifyError> {
        let hybrid = self.hybrid.clone();
        let t = track.clone();

        let stream = self
            .runtime
            .block_on(async move { hybrid.resolve_stream(&t).await })
            .map_err(|e| SoftifyError::ResolutionError { msg: e })?;

        // Record successful playback session
        let _ = self.db.record_playback(&track, &stream);

        Ok(stream)
    }

    /// Retrieves synchronized or plain lyrics from LRCLIB / JioSaavn.
    pub fn get_lyrics(&self, title: String, artist: String) -> Option<String> {
        let lyrics = self.lyrics.clone();
        self.runtime
            .block_on(async move { lyrics.resolve_lyrics(&title, &artist, None).await })
    }

    /// Downloads studio-grade audio stream to the given destination directory and returns the absolute file path.
    pub fn download_track(
        &self,
        track: TrackMetadata,
        destination_dir: String,
    ) -> Result<String, SoftifyError> {
        let stream = self.resolve_stream(track.clone())?;
        let filename = get_track_filename(&track.title, &track.artist, &stream.container);
        let dest_path: PathBuf = Path::new(&destination_dir).join(&filename);

        let client = self.http_client.clone();
        let stream_url = stream.stream_url.clone();
        let target_path = dest_path.clone();

        self.runtime
            .block_on(async move {
                download_audio_stream(&client, &stream_url, &target_path).await
            })
            .map_err(|e| SoftifyError::DownloadError { msg: e })?;

        Ok(dest_path.to_string_lossy().to_string())
    }

    /// Checks if a file path is offline-ready and fully downloaded.
    pub fn is_file_offline(&self, file_path: String) -> bool {
        is_offline_ready(Path::new(&file_path))
    }

    /// Searches cached catalog via SQLite FTS5 full-text search.
    pub fn search_cache(&self, query: String, limit: u32) -> Vec<TrackMetadata> {
        self.db
            .search_tracks_fts(&query, limit as usize)
            .unwrap_or_default()
    }

    /// Retrieves recently played tracks.
    pub fn get_playback_history(&self, limit: u32) -> Vec<TrackMetadata> {
        self.db
            .get_playback_history(limit as usize)
            .unwrap_or_default()
    }
}

impl SoftifyEngine for SoftifyEngineService {
    fn search(&self, query: String) -> Vec<TrackMetadata> {
        self.search(query)
    }

    fn resolve_stream(&self, track: TrackMetadata) -> Result<StreamResult, String> {
        SoftifyEngineService::resolve_stream(self, track).map_err(|e| e.to_string())
    }

    fn get_lyrics(&self, title: String, artist: String) -> Option<String> {
        self.get_lyrics(title, artist)
    }
}
