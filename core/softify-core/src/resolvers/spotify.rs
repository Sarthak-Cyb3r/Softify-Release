use crate::models::TrackMetadata;
use crate::normalizer::normalize_unicode_spaces;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const SPOTIFY_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";
const PATHFINDER_QUERY_HASH: &str =
    "12c1a6f114c0a50157303c72b8314ff5a7f2924fa68ff212e3e53e4a9e52ce64";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnonymousTokenResponse {
    #[serde(rename = "clientId")]
    pub client_id: Option<String>,
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "accessTokenExpirationTimestampMs")]
    pub access_token_expiration_timestamp_ms: u64,
    #[serde(rename = "isAnonymous", default)]
    pub is_anonymous: bool,
}

#[derive(Clone, Debug)]
pub struct SpotifyTokenCache {
    pub access_token: String,
    pub expires_at_ms: u64,
}

#[derive(Clone, Debug)]
pub struct SpotifyResolver {
    client: Client,
    token_cache: Arc<RwLock<Option<SpotifyTokenCache>>>,
}

impl SpotifyResolver {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(SPOTIFY_USER_AGENT)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Obtains a valid anonymous access token, caching in memory and auto-refreshing
    /// 5 minutes (300,000 ms) before expiration.
    pub async fn get_anonymous_token(&self) -> Result<String, String> {
        let current_time_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("System time error: {e}"))?
            .as_millis() as u64;

        // Check cached token
        {
            let cache = self.token_cache.read().await;
            if let Some(cached) = &*cache {
                // Buffer of 5 minutes (300,000 ms)
                if cached.expires_at_ms > current_time_ms + 300_000 {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Acquire write lock to refresh token
        let mut cache = self.token_cache.write().await;
        // Double-check after lock
        if let Some(cached) = &*cache {
            if cached.expires_at_ms > current_time_ms + 300_000 {
                return Ok(cached.access_token.clone());
            }
        }

        let token_url =
            "https://open.spotify.com/get_access_token?reason=transport&productType=web_player";
        let resp = self
            .client
            .get(token_url)
            .header("User-Agent", SPOTIFY_USER_AGENT)
            .send()
            .await
            .map_err(|e| format!("Failed to request Spotify anonymous token: {e}"))?;

        let token_data: AnonymousTokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Spotify token JSON: {e}"))?;

        *cache = Some(SpotifyTokenCache {
            access_token: token_data.access_token.clone(),
            expires_at_ms: token_data.access_token_expiration_timestamp_ms,
        });

        Ok(token_data.access_token)
    }

    /// Executes Spotify Pathfinder GraphQL searchDesktop query.
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<TrackMetadata>, String> {
        let access_token = self.get_anonymous_token().await?;

        let variables = serde_json::json!({
            "searchTerm": query,
            "offset": 0,
            "limit": limit.min(50),
            "numberOfTopResults": 5
        });

        let extensions = serde_json::json!({
            "persistedQuery": {
                "version": 1,
                "sha256Hash": PATHFINDER_QUERY_HASH
            }
        });

        let pathfinder_url = format!(
            "https://api-partner.spotify.com/pathfinder/v1/query?operationName=searchDesktop&variables={}&extensions={}",
            urlencoding_encode(&variables.to_string()),
            urlencoding_encode(&extensions.to_string())
        );

        let resp = self
            .client
            .get(&pathfinder_url)
            .header("User-Agent", SPOTIFY_USER_AGENT)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| format!("Spotify Pathfinder request failed: {e}"))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("Spotify Pathfinder body read failed: {e}"))?;

        Self::parse_pathfinder_search_response(&text)
    }

    /// Parses Spotify Pathfinder search response JSON and normalizes Unicode whitespace.
    pub fn parse_pathfinder_search_response(json_str: &str) -> Result<Vec<TrackMetadata>, String> {
        let root: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid Pathfinder JSON: {e}"))?;

        let items = root
            .pointer("/data/searchV2/tracksV2/items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "No /data/searchV2/tracksV2/items found in Pathfinder response".to_string())?;

        let mut tracks = Vec::with_capacity(items.len());

        for item in items {
            let track_data = match item.pointer("/item/data") {
                Some(d) => d,
                None => continue,
            };

            let uri = track_data
                .get("uri")
                .and_then(|u| u.as_str())
                .unwrap_or_default();
            let id = uri.strip_prefix("spotify:track:").unwrap_or(uri).to_string();

            let raw_title = track_data
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or_default();
            // Normalization Law: mandatory \u{00A0} and Unicode whitespace replacement
            let title = normalize_unicode_spaces(raw_title);

            // Extract artist names
            let mut artists = Vec::new();
            if let Some(artist_items) = track_data.pointer("/artists/items").and_then(|a| a.as_array()) {
                for a in artist_items {
                    if let Some(name) = a.pointer("/profile/name").and_then(|n| n.as_str()) {
                        let normalized_name = normalize_unicode_spaces(name);
                        if !normalized_name.is_empty() {
                            artists.push(normalized_name);
                        }
                    }
                }
            }

            let artist = if artists.is_empty() {
                "Unknown Artist".to_string()
            } else {
                artists.join(", ")
            };

            // Extract album name and cover art
            let album_name = track_data
                .pointer("/albumOfTrack/name")
                .and_then(|a| a.as_str())
                .map(normalize_unicode_spaces);

            let cover_url = track_data
                .pointer("/albumOfTrack/coverArt/sources")
                .and_then(|s| s.as_array())
                .and_then(|sources| sources.first())
                .and_then(|src| src.get("url"))
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());

            let duration_ms = track_data
                .pointer("/trackDuration/totalMilliseconds")
                .and_then(|d| d.as_u64())
                .unwrap_or(0);
            let duration_seconds = (duration_ms / 1000) as u32;

            tracks.push(TrackMetadata {
                id,
                title,
                artist,
                album: album_name,
                duration_seconds,
                cover_url,
            });
        }

        Ok(tracks)
    }
}

fn urlencoding_encode(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pathfinder_search_response_with_unicode_normalization() {
        let sample_json = r#"{
            "data": {
                "searchV2": {
                    "tracksV2": {
                        "items": [
                            {
                                "item": {
                                    "data": {
                                        "uri": "spotify:track:0VjIjW4GlUZAMYd2vXMi3b",
                                        "name": "Blinding\u00A0Lights",
                                        "trackDuration": {
                                            "totalMilliseconds": 200040
                                        },
                                        "artists": {
                                            "items": [
                                                {
                                                    "profile": {
                                                        "name": "The\u00A0Weeknd"
                                                    }
                                                }
                                            ]
                                        },
                                        "albumOfTrack": {
                                            "name": "After Hours",
                                            "coverArt": {
                                                "sources": [
                                                    {
                                                        "url": "https://i.scdn.co/image/ab67616d0000b2738863bc11d2aa12b54f5aeb36",
                                                        "width": 640,
                                                        "height": 640
                                                    }
                                                ]
                                            }
                                        }
                                    }
                                }
                            }
                        ]
                    }
                }
            }
        }"#;

        let tracks = SpotifyResolver::parse_pathfinder_search_response(sample_json)
            .expect("Should parse pathfinder response");

        assert_eq!(tracks.len(), 1);
        let track = &tracks[0];
        assert_eq!(track.id, "0VjIjW4GlUZAMYd2vXMi3b");
        // Check mandatory non-breaking space replacement
        assert_eq!(track.title, "Blinding Lights");
        assert_eq!(track.artist, "The Weeknd");
        assert_eq!(track.duration_seconds, 200);
        assert_eq!(track.album, Some("After Hours".to_string()));
        assert!(track.cover_url.is_some());
    }
}
