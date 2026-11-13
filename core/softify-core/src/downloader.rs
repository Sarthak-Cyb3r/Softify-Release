use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Sanitizes a string for use as a filesystem path on Linux, macOS, Windows, and Android.
pub fn sanitize_filename(name: &str) -> String {
    let forbidden = ['/', '\\', '<', '>', ':', '"', '|', '?', '*', '\0'];
    let mut sanitized = String::with_capacity(name.len());

    for ch in name.chars() {
        if forbidden.contains(&ch) {
            sanitized.push('_');
        } else {
            sanitized.push(ch);
        }
    }

    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        "track".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Generates standardized offline audio filename: "{Artist} - {Title}.m4a"
pub fn get_track_filename(title: &str, artist: &str, container: &str) -> String {
    let ext = if container.trim().is_empty() {
        "m4a"
    } else {
        container.trim()
    };
    let base = format!("{} - {}", artist, title);
    format!("{}.{}", sanitize_filename(&base), ext)
}

/// Downloads audio stream chunk-by-chunk to a temporary `.part` file, then atomically renames to target.
pub async fn download_audio_stream(
    client: &Client,
    stream_url: &str,
    destination: &Path,
) -> Result<u64, String> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create download directory: {e}"))?;
    }

    let mut part_path: PathBuf = destination.to_path_buf();
    let file_name = part_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("track.m4a");
    part_path.set_file_name(format!("{}.part", file_name));

    let mut response = client
        .get(stream_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("Stream HTTP request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with HTTP status: {}",
            response.status()
        ));
    }

    let mut file = File::create(&part_path)
        .map_err(|e| format!("Failed to create part file '{:?}': {e}", part_path))?;

    let mut total_bytes: u64 = 0;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("Error while streaming audio chunk: {e}"))?
    {
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write to file: {e}"))?;
        total_bytes += chunk.len() as u64;
    }

    file.flush()
        .map_err(|e| format!("Failed to flush written audio data: {e}"))?;
    drop(file);

    // Atomically rename .part to final destination
    std::fs::rename(&part_path, destination)
        .map_err(|e| format!("Failed to rename part file to final audio file: {e}"))?;

    Ok(total_bytes)
}

/// Checks if an offline track file exists and is non-empty.
pub fn is_offline_ready(path: &Path) -> bool {
    if let Ok(metadata) = std::fs::metadata(path) {
        metadata.is_file() && metadata.len() > 0
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            sanitize_filename("AC/DC: Highway to Hell?"),
            "AC_DC_ Highway to Hell_"
        );
        assert_eq!(
            get_track_filename("Blinding Lights", "The Weeknd", "m4a"),
            "The Weeknd - Blinding Lights.m4a"
        );
    }
}
