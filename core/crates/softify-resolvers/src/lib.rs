//! Stream and metadata resolvers for Softify Next-Gen.
//!
//! Provides clean, reverse-engineered catalog resolvers for:
//! - JioSaavn 320 Kbps CD-quality studio master streams
//! - Spotify Pathfinder GraphQL metadata search & anonymous tokens
//! - Spotify Playlist and Album importer with track pagination
//! - YouTube Music InnerTube (`WEB_REMIX`) resolver with Topic channel scoring
//! - Direct YouTube Video URL audio extractor
//! - Synchronized lyrics via LRCLIB + JioSaavn with Romanization helper
//! - Hybrid fallback coordinator

pub mod hybrid;
pub mod jiosaavn;
pub mod lyrics;
pub mod spotify;
pub mod spotify_importer;
pub mod youtube;

pub use hybrid::*;
pub use jiosaavn::*;
pub use lyrics::*;
pub use spotify::*;
pub use spotify_importer::*;
pub use youtube::*;
