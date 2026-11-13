use crate::spotify::SpotifyResolver;
use reqwest::Client;
use serde_json::Value;
use softify_crypto::normalize_unicode_spaces;
use softify_models::TrackMetadata;

const SPOTIFY_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

/// Target type and identifier for Spotify importing operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportTarget {
    Playlist(String),
    Album(String),
}

/// Imported collection containing catalog metadata and paginated tracks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedCollection {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub total_tracks: usize,
    pub tracks: Vec<TrackMetadata>,
}

#[derive(Clone, Debug)]
pub struct SpotifyImporter {
    spotify: SpotifyResolver,
    client: Client,
}

impl SpotifyImporter {
    pub fn new(spotify: SpotifyResolver) -> Self {
        let client = Client::builder()
            .user_agent(SPOTIFY_USER_AGENT)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { spotify, client }
    }

    /// Parses URL, URI, or ID string into an `ImportTarget` (Playlist or Album).
    pub fn parse_target(input: &str) -> Result<ImportTarget, String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err("Input Spotify URL or URI cannot be empty".to_string());
        }

        // Spotify URI: spotify:playlist:ID or spotify:album:ID
        if let Some(rest) = trimmed.strip_prefix("spotify:playlist:") {
            let id = clean_spotify_id(rest);
            if !id.is_empty() {
                return Ok(ImportTarget::Playlist(id.to_string()));
            }
        }
        if let Some(rest) = trimmed.strip_prefix("spotify:album:") {
            let id = clean_spotify_id(rest);
            if !id.is_empty() {
                return Ok(ImportTarget::Album(id.to_string()));
            }
        }

        // Spotify Web URL: https://open.spotify.com/playlist/ID or https://open.spotify.com/album/ID
        if let Some(idx) = trimmed.find("/playlist/") {
            let rest = &trimmed[idx + "/playlist/".len()..];
            let id = clean_spotify_id(rest);
            if !id.is_empty() {
                return Ok(ImportTarget::Playlist(id.to_string()));
            }
        }

        if let Some(idx) = trimmed.find("/album/") {
            let rest = &trimmed[idx + "/album/".len()..];
            let id = clean_spotify_id(rest);
            if !id.is_empty() {
                return Ok(ImportTarget::Album(id.to_string()));
            }
        }

        Err(format!(
            "Unable to parse Spotify URL or URI: '{}'. Expected playlist or album URL/URI.",
            input
        ))
    }

    /// Imports a collection (playlist or album) by URL, URI, or parsed target.
    pub async fn import(&self, target_input: &str) -> Result<ImportedCollection, String> {
        let target = Self::parse_target(target_input)?;
        match target {
            ImportTarget::Playlist(id) => self.import_playlist(&id).await,
            ImportTarget::Album(id) => self.import_album(&id).await,
        }
    }

    /// Imports an entire Spotify playlist, paginating all tracks.
    pub async fn import_playlist(&self, playlist_id: &str) -> Result<ImportedCollection, String> {
        let token = self.spotify.get_anonymous_token().await?;

        // 1. Fetch initial playlist metadata and first batch of tracks
        let url = format!(
            "https://api.spotify.com/v1/playlists/{}?market=US",
            playlist_id
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", SPOTIFY_USER_AGENT)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Spotify playlist: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Spotify playlist API error {status}: {body}"));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read playlist body: {e}"))?;

        let (title, description, cover_url, total_tracks, mut tracks, next_page) =
            Self::parse_playlist_meta_and_tracks(&body)?;

        // 2. Paginate remaining tracks if any
        let mut next_url = next_page;
        while let Some(url) = next_url {
            let resp = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {token}"))
                .header("User-Agent", SPOTIFY_USER_AGENT)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch playlist page: {e}"))?;

            if !resp.status().is_success() {
                break;
            }

            let page_body = resp.text().await.unwrap_or_default();
            let (page_tracks, next) = Self::parse_playlist_page(&page_body)?;
            if page_tracks.is_empty() {
                break;
            }
            tracks.extend(page_tracks);
            next_url = next;
        }

        Ok(ImportedCollection {
            id: playlist_id.to_string(),
            title,
            description,
            cover_url,
            total_tracks: total_tracks.max(tracks.len()),
            tracks,
        })
    }

    /// Imports an entire Spotify album, paginating all tracks.
    pub async fn import_album(&self, album_id: &str) -> Result<ImportedCollection, String> {
        let token = self.spotify.get_anonymous_token().await?;

        // 1. Fetch initial album metadata and first batch of tracks
        let url = format!("https://api.spotify.com/v1/albums/{}?market=US", album_id);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("User-Agent", SPOTIFY_USER_AGENT)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch Spotify album: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Spotify album API error {status}: {body}"));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read album body: {e}"))?;

        let (title, description, cover_url, total_tracks, mut tracks, next_page) =
            Self::parse_album_meta_and_tracks(&body)?;

        // 2. Paginate remaining album tracks if any
        let mut next_url = next_page;
        while let Some(url) = next_url {
            let resp = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {token}"))
                .header("User-Agent", SPOTIFY_USER_AGENT)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch album page: {e}"))?;

            if !resp.status().is_success() {
                break;
            }

            let page_body = resp.text().await.unwrap_or_default();
            let (page_tracks, next) =
                Self::parse_album_page(&title, cover_url.as_deref(), &page_body)?;
            if page_tracks.is_empty() {
                break;
            }
            tracks.extend(page_tracks);
            next_url = next;
        }

        Ok(ImportedCollection {
            id: album_id.to_string(),
            title,
            description,
            cover_url,
            total_tracks: total_tracks.max(tracks.len()),
            tracks,
        })
    }

    /// Parses initial playlist JSON body into metadata, first track batch, and optional next URL.
    pub fn parse_playlist_meta_and_tracks(
        json_str: &str,
    ) -> Result<
        (
            String,
            Option<String>,
            Option<String>,
            usize,
            Vec<TrackMetadata>,
            Option<String>,
        ),
        String,
    > {
        let root: Value =
            serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {e}"))?;

        let title = root
            .get("name")
            .and_then(|n| n.as_str())
            .map(normalize_unicode_spaces)
            .unwrap_or_else(|| "Untitled Playlist".to_string());

        let description = root
            .get("description")
            .and_then(|d| d.as_str())
            .map(normalize_unicode_spaces)
            .filter(|d| !d.is_empty());

        let cover_url = root
            .get("images")
            .and_then(|imgs| imgs.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        let tracks_obj = root
            .get("tracks")
            .ok_or_else(|| "No 'tracks' object in playlist response".to_string())?;

        let total = tracks_obj
            .get("total")
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as usize;

        let next = tracks_obj
            .get("next")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        let items = tracks_obj
            .get("items")
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default();

        let tracks = Self::extract_tracks_from_items(&items);

        Ok((title, description, cover_url, total, tracks, next))
    }

    /// Parses paginated playlist page response containing `items` and `next`.
    pub fn parse_playlist_page(
        json_str: &str,
    ) -> Result<(Vec<TrackMetadata>, Option<String>), String> {
        let root: Value =
            serde_json::from_str(json_str).map_err(|e| format!("Invalid page JSON: {e}"))?;

        let next = root
            .get("next")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        let items = root
            .get("items")
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default();

        let tracks = Self::extract_tracks_from_items(&items);
        Ok((tracks, next))
    }

    /// Parses initial album JSON body into metadata, first track batch, and optional next URL.
    pub fn parse_album_meta_and_tracks(
        json_str: &str,
    ) -> Result<
        (
            String,
            Option<String>,
            Option<String>,
            usize,
            Vec<TrackMetadata>,
            Option<String>,
        ),
        String,
    > {
        let root: Value =
            serde_json::from_str(json_str).map_err(|e| format!("Invalid album JSON: {e}"))?;

        let title = root
            .get("name")
            .and_then(|n| n.as_str())
            .map(normalize_unicode_spaces)
            .unwrap_or_else(|| "Untitled Album".to_string());

        let artists = root
            .get("artists")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|artist| artist.get("name").and_then(|n| n.as_str()))
                    .map(normalize_unicode_spaces)
                    .collect::<Vec<_>>()
                    .join(", ")
            });

        let description = artists.filter(|a| !a.is_empty());

        let cover_url = root
            .get("images")
            .and_then(|imgs| imgs.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        let tracks_obj = root
            .get("tracks")
            .ok_or_else(|| "No 'tracks' object in album response".to_string())?;

        let total = tracks_obj
            .get("total")
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as usize;

        let next = tracks_obj
            .get("next")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        let items = tracks_obj
            .get("items")
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default();

        let tracks = Self::extract_album_tracks_from_items(&items, &title, cover_url.as_deref());

        Ok((title, description, cover_url, total, tracks, next))
    }

    /// Parses paginated album page response.
    pub fn parse_album_page(
        album_title: &str,
        cover_url: Option<&str>,
        json_str: &str,
    ) -> Result<(Vec<TrackMetadata>, Option<String>), String> {
        let root: Value =
            serde_json::from_str(json_str).map_err(|e| format!("Invalid album page JSON: {e}"))?;

        let next = root
            .get("next")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        let items = root
            .get("items")
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default();

        let tracks = Self::extract_album_tracks_from_items(&items, album_title, cover_url);
        Ok((tracks, next))
    }

    fn extract_tracks_from_items(items: &[Value]) -> Vec<TrackMetadata> {
        let mut tracks = Vec::with_capacity(items.len());

        for item in items {
            let track_obj = match item.get("track") {
                Some(t) if t.is_object() => t,
                _ => item,
            };

            let id = track_obj
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or_default();
            if id.is_empty() {
                continue;
            }

            let raw_title = track_obj
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or_default();
            let title = normalize_unicode_spaces(raw_title);

            let artists = track_obj
                .get("artists")
                .and_then(|a| a.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|artist| artist.get("name").and_then(|n| n.as_str()))
                        .map(normalize_unicode_spaces)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "Unknown Artist".to_string());

            let album_name = track_obj
                .get("album")
                .and_then(|alb| alb.get("name"))
                .and_then(|n| n.as_str())
                .map(normalize_unicode_spaces);

            let cover_url = track_obj
                .get("album")
                .and_then(|alb| alb.get("images"))
                .and_then(|imgs| imgs.as_array())
                .and_then(|arr| arr.first())
                .and_then(|first| first.get("url"))
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());

            let duration_ms = track_obj
                .get("duration_ms")
                .and_then(|d| d.as_u64())
                .unwrap_or(0);

            tracks.push(TrackMetadata {
                id: id.to_string(),
                title,
                artist: artists,
                album: album_name,
                duration_seconds: (duration_ms / 1000) as u32,
                cover_url,
            });
        }

        tracks
    }

    fn extract_album_tracks_from_items(
        items: &[Value],
        album_title: &str,
        cover_url: Option<&str>,
    ) -> Vec<TrackMetadata> {
        let mut tracks = Vec::with_capacity(items.len());

        for item in items {
            let id = item.get("id").and_then(|i| i.as_str()).unwrap_or_default();
            if id.is_empty() {
                continue;
            }

            let raw_title = item
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or_default();
            let title = normalize_unicode_spaces(raw_title);

            let artists = item
                .get("artists")
                .and_then(|a| a.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|artist| artist.get("name").and_then(|n| n.as_str()))
                        .map(normalize_unicode_spaces)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "Unknown Artist".to_string());

            let duration_ms = item
                .get("duration_ms")
                .and_then(|d| d.as_u64())
                .unwrap_or(0);

            tracks.push(TrackMetadata {
                id: id.to_string(),
                title,
                artist: artists,
                album: Some(album_title.to_string()),
                duration_seconds: (duration_ms / 1000) as u32,
                cover_url: cover_url.map(|s| s.to_string()),
            });
        }

        tracks
    }
}

fn clean_spotify_id(raw: &str) -> &str {
    let without_slash = raw.trim_start_matches('/');
    let id_part = match without_slash.find('?') {
        Some(idx) => &without_slash[..idx],
        None => without_slash,
    };
    id_part.trim_matches('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_urls_and_uris() {
        // Web URLs
        assert_eq!(
            SpotifyImporter::parse_target(
                "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=abcdef"
            )
            .unwrap(),
            ImportTarget::Playlist("37i9dQZF1DXcBWIGoYBM5M".to_string())
        );

        assert_eq!(
            SpotifyImporter::parse_target("https://open.spotify.com/album/4yP0hdKO0BsV870isFUUYN")
                .unwrap(),
            ImportTarget::Album("4yP0hdKO0BsV870isFUUYN".to_string())
        );

        // URIs
        assert_eq!(
            SpotifyImporter::parse_target("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M").unwrap(),
            ImportTarget::Playlist("37i9dQZF1DXcBWIGoYBM5M".to_string())
        );

        assert_eq!(
            SpotifyImporter::parse_target("spotify:album:4yP0hdKO0BsV870isFUUYN").unwrap(),
            ImportTarget::Album("4yP0hdKO0BsV870isFUUYN".to_string())
        );

        // Invalid
        assert!(SpotifyImporter::parse_target("https://youtube.com/watch?v=123").is_err());
    }

    #[test]
    fn test_parse_playlist_meta_and_tracks() {
        let json = r#"{
            "name": "Today's\u00A0Top\u00A0Hits",
            "description": "Jung Kook is on top of the Hottest 50!",
            "images": [
                { "url": "https://i.scdn.co/image/playlist_cover.jpg" }
            ],
            "tracks": {
                "total": 50,
                "next": "https://api.spotify.com/v1/playlists/123/tracks?offset=50&limit=50",
                "items": [
                    {
                        "track": {
                            "id": "trk_1",
                            "name": "Seven\u00A0(feat. Latto)",
                            "duration_ms": 184400,
                            "artists": [
                                { "name": "Jung\u00A0Kook" },
                                { "name": "Latto" }
                            ],
                            "album": {
                                "name": "Seven",
                                "images": [
                                    { "url": "https://i.scdn.co/image/seven_cover.jpg" }
                                ]
                            }
                        }
                    }
                ]
            }
        }"#;

        let (title, desc, cover, total, tracks, next) =
            SpotifyImporter::parse_playlist_meta_and_tracks(json).unwrap();

        assert_eq!(title, "Today's Top Hits");
        assert_eq!(
            desc.as_deref(),
            Some("Jung Kook is on top of the Hottest 50!")
        );
        assert_eq!(
            cover.as_deref(),
            Some("https://i.scdn.co/image/playlist_cover.jpg")
        );
        assert_eq!(total, 50);
        assert_eq!(
            next.as_deref(),
            Some("https://api.spotify.com/v1/playlists/123/tracks?offset=50&limit=50")
        );
        assert_eq!(tracks.len(), 1);

        let t = &tracks[0];
        assert_eq!(t.id, "trk_1");
        assert_eq!(t.title, "Seven (feat. Latto)");
        assert_eq!(t.artist, "Jung Kook, Latto");
        assert_eq!(t.album.as_deref(), Some("Seven"));
        assert_eq!(t.duration_seconds, 184);
        assert_eq!(
            t.cover_url.as_deref(),
            Some("https://i.scdn.co/image/seven_cover.jpg")
        );
    }

    #[test]
    fn test_parse_album_meta_and_tracks() {
        let json = r#"{
            "name": "After Hours",
            "artists": [
                { "name": "The Weeknd" }
            ],
            "images": [
                { "url": "https://i.scdn.co/image/after_hours.jpg" }
            ],
            "tracks": {
                "total": 14,
                "next": null,
                "items": [
                    {
                        "id": "ah_trk_1",
                        "name": "Alone Again",
                        "duration_ms": 250000,
                        "artists": [
                            { "name": "The Weeknd" }
                        ]
                    }
                ]
            }
        }"#;

        let (title, desc, cover, total, tracks, next) =
            SpotifyImporter::parse_album_meta_and_tracks(json).unwrap();

        assert_eq!(title, "After Hours");
        assert_eq!(desc.as_deref(), Some("The Weeknd"));
        assert_eq!(
            cover.as_deref(),
            Some("https://i.scdn.co/image/after_hours.jpg")
        );
        assert_eq!(total, 14);
        assert_eq!(next, None);
        assert_eq!(tracks.len(), 1);

        let t = &tracks[0];
        assert_eq!(t.id, "ah_trk_1");
        assert_eq!(t.title, "Alone Again");
        assert_eq!(t.artist, "The Weeknd");
        assert_eq!(t.album.as_deref(), Some("After Hours"));
        assert_eq!(t.duration_seconds, 250);
        assert_eq!(
            t.cover_url.as_deref(),
            Some("https://i.scdn.co/image/after_hours.jpg")
        );
    }
}
