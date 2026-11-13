use clap::{Parser, Subcommand};
use softify_core::crypto::des::decrypt_jiosaavn_url;
use softify_core::models::TrackMetadata;
use softify_core::SoftifyEngineService;

#[derive(Parser)]
#[command(name = "softify-cli")]
#[command(about = "Softify Next-Gen CLI Runner for Stream Resolution & Verification", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for music tracks across Spotify Pathfinder & JioSaavn
    Search {
        /// Search query (e.g. "Blinding Lights")
        query: String,
    },
    /// Resolve 320 Kbps stream for a given song title and artist
    Resolve {
        /// Song title (e.g. "Blinding Lights")
        title: String,
        /// Artist name (e.g. "The Weeknd")
        artist: String,
        /// Track duration in seconds (optional)
        #[arg(short, long)]
        duration: Option<u32>,
    },
    /// Decrypt a JioSaavn encrypted_media_url directly and output 320k stream
    Decrypt {
        /// Base64 encrypted media URL string
        ciphertext: String,
    },
    /// Query local SQLite FTS5 catalog cache
    CacheSearch {
        /// Search query for local database cache
        query: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let engine = SoftifyEngineService::new(None)
        .map_err(|e| format!("Engine initialization failed: {e}"))?;

    match cli.command {
        Commands::Search { query } => {
            println!("🔍 Searching for: \"{}\"...", query);
            let tracks = engine.search(query);
            if tracks.is_empty() {
                println!("No tracks found.");
            } else {
                println!("\nFound {} tracks:", tracks.len());
                println!("{:-<80}", "");
                for (idx, track) in tracks.iter().enumerate() {
                    let album = track.album.as_deref().unwrap_or("Single");
                    println!(
                        "[{:>2}] {} - {} ({}) [{}s]",
                        idx + 1,
                        track.artist,
                        track.title,
                        album,
                        track.duration_seconds
                    );
                }
                println!("{:-<80}", "");
            }
        }
        Commands::Resolve {
            title,
            artist,
            duration,
        } => {
            println!("🎵 Resolving stream for: \"{}\" by \"{}\"...", title, artist);
            let track = TrackMetadata {
                id: "cli_query".to_string(),
                title: title.clone(),
                artist: artist.clone(),
                album: None,
                duration_seconds: duration.unwrap_or(0),
                cover_url: None,
            };

            match engine.resolve_stream(track) {
                Ok(stream) => {
                    println!("\n✨ Stream Successfully Resolved!");
                    println!("{:-<80}", "");
                    println!("Provider:  {}", stream.provider);
                    println!("Bitrate:   {} kbps", stream.bitrate / 1000);
                    println!("Container: {}", stream.container);
                    println!("Codec:     {}", stream.codec);
                    println!("Stream URL: {}", stream.stream_url);
                    println!("{:-<80}", "");
                }
                Err(err) => {
                    eprintln!("\n❌ Resolution Failed: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Commands::Decrypt { ciphertext } => {
            println!("🔐 Decrypting JioSaavn ciphertext...");
            match decrypt_jiosaavn_url(&ciphertext) {
                Ok(url_320) => {
                    println!("\n✨ Successfully Decrypted & Upgraded to 320 Kbps:");
                    println!("Stream URL: {}", url_320);
                }
                Err(err) => {
                    eprintln!("\n❌ Decryption Failed: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Commands::CacheSearch { query } => {
            println!("📂 Querying SQLite FTS5 cache for: \"{}\"...", query);
            let tracks = engine.search_cache(query, 20);
            if tracks.is_empty() {
                println!("No cached tracks found.");
            } else {
                for (idx, track) in tracks.iter().enumerate() {
                    println!(
                        "[{:>2}] {} - {} (ID: {})",
                        idx + 1,
                        track.artist,
                        track.title,
                        track.id
                    );
                }
            }
        }
    }

    Ok(())
}
