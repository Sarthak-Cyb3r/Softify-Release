use crate::crypto::des::decrypt_jiosaavn_url;
use crate::models::{StreamResult, TrackMetadata};
use crate::normalizer::{
    clean_title, is_disqualified_variant, normalize_unicode_spaces, score_duration_proximity,
    token_similarity,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const JIOSAAVN_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

#[derive(Clone, Debug)]
pub struct JioSaavnResolver {
    client: Client,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JioSaavnArtist {
    pub id: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct JioSaavnArtistMap {
    #[serde(default)]
    pub primary_artists: Vec<JioSaavnArtist>,
    #[serde(default)]
    pub featured_artists: Vec<JioSaavnArtist>,
    #[serde(default)]
    pub artists: Vec<JioSaavnArtist>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct JioSaavnMoreInfo {
    pub album: Option<String>,
    pub encrypted_media_url: Option<String>,
    #[serde(rename = "artistMap", default)]
    pub artist_map: Option<JioSaavnArtistMap>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JioSaavnSong {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub duration: Option<serde_json::Value>,
    pub image: Option<String>,
    #[serde(default)]
    pub more_info: Option<JioSaavnMoreInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JioSaavnSearchResponse {
    #[serde(default)]
    pub results: Vec<JioSaavnSong>,
}

impl JioSaavnResolver {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(JIOSAAVN_USER_AGENT)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    /// Searches JioSaavn catalog and returns clean TrackMetadata list.
    pub async fn search(&self, query: &str) -> Result<Vec<TrackMetadata>, String> {
        let encoded_query = urlencoding_encode(query);
        let url = format!(
            "https://www.jiosaavn.com/api.php?__call=search.getResults&_format=json&_marker=0&api_version=4&ctx=web6dot0&n=15&p=1&q={}",
            encoded_query
        );

        let resp = self
            .client
            .get(&url)
            .header("User-Agent", JIOSAAVN_USER_AGENT)
            .send()
            .await
            .map_err(|e| format!("JioSaavn HTTP request failed: {e}"))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("JioSaavn response text failed: {e}"))?;

        Self::parse_search_results(&text)
    }

    /// Resolves high-fidelity 320 Kbps stream for a given track metadata.
    pub async fn resolve_stream(&self, track: &TrackMetadata) -> Result<StreamResult, String> {
        let query = format!("{} {}", track.title, track.artist);
        let encoded_query = urlencoding_encode(&query);
        let url = format!(
            "https://www.jiosaavn.com/api.php?__call=search.getResults&_format=json&_marker=0&api_version=4&ctx=web6dot0&n=15&p=1&q={}",
            encoded_query
        );

        let resp = self
            .client
            .get(&url)
            .header("User-Agent", JIOSAAVN_USER_AGENT)
            .send()
            .await
            .map_err(|e| format!("JioSaavn search request failed: {e}"))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read JioSaavn body: {e}"))?;

        let parsed: JioSaavnSearchResponse = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse JioSaavn search response: {e}"))?;

        let best_candidate = Self::find_best_candidate(&parsed.results, track, &query)?;

        let encrypted_url = best_candidate
            .more_info
            .as_ref()
            .and_then(|m| m.encrypted_media_url.as_ref())
            .ok_or_else(|| "Candidate lacks encrypted_media_url".to_string())?;

        let stream_url_320 = decrypt_jiosaavn_url(encrypted_url)?;

        Ok(StreamResult {
            stream_url: stream_url_320,
            bitrate: 320000,
            container: "m4a".to_string(),
            codec: "aac".to_string(),
            provider: "jiosaavn_studio_320k".to_string(),
        })
    }

    /// Parses search response text into canonical TrackMetadata.
    pub fn parse_search_results(json_str: &str) -> Result<Vec<TrackMetadata>, String> {
        let parsed: JioSaavnSearchResponse = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse JioSaavn search results JSON: {e}"))?;

        let mut tracks = Vec::with_capacity(parsed.results.len());
        for song in parsed.results {
            let artist = Self::extract_artists(&song);
            let duration = Self::parse_duration(&song.duration);
            let album = song
                .more_info
                .as_ref()
                .and_then(|m| m.album.clone())
                .filter(|a| !a.trim().is_empty());

            let cover = song.image.map(|img| {
                // Upgrade cover image resolution to 500x500
                img.replace("150x150", "500x500")
                    .replace("50x50", "500x500")
            });

            tracks.push(TrackMetadata {
                id: song.id,
                title: decode_html_entities(&song.title),
                artist,
                album,
                duration_seconds: duration,
                cover_url: cover,
            });
        }

        Ok(tracks)
    }

    /// Filters and scores candidates to find the exact studio track match.
    pub fn find_best_candidate<'a>(
        candidates: &'a [JioSaavnSong],
        target: &TrackMetadata,
        user_query: &str,
    ) -> Result<&'a JioSaavnSong, String> {
        let mut scored_candidates: Vec<(&'a JioSaavnSong, i32)> = Vec::new();

        for song in candidates {
            // Disqualify if no encrypted_media_url
            let has_media = song
                .more_info
                .as_ref()
                .and_then(|m| m.encrypted_media_url.as_ref())
                .is_some();
            if !has_media {
                continue;
            }

            // Rule: Actor Disqualification
            // Discard any entry in artistMap.artists where role == "starring"
            if let Some(more) = &song.more_info {
                if let Some(map) = &more.artist_map {
                    let has_starring = map
                        .artists
                        .iter()
                        .any(|a| a.role.as_deref() == Some("starring"));
                    if has_starring {
                        continue;
                    }
                }
            }

            let candidate_title = decode_html_entities(&song.title);
            let candidate_artist = Self::extract_artists(song);

            // Rule: Variant Disqualification
            if is_disqualified_variant(&candidate_title, user_query) {
                continue;
            }

            // Duration Proximity Rule
            let candidate_duration = Self::parse_duration(&song.duration);
            let duration_score = if target.duration_seconds > 0 && candidate_duration > 0 {
                match score_duration_proximity(candidate_duration, target.duration_seconds) {
                    Some(score) => score,
                    None => continue, // Disqualified: duration discrepancy > 90s
                }
            } else {
                0
            };

            // Title & Artist Similarity
            let cleaned_target_title = clean_title(&target.title);
            let cleaned_cand_title = clean_title(&candidate_title);

            let title_sim = token_similarity(&cleaned_cand_title, &cleaned_target_title);
            let artist_sim = token_similarity(&candidate_artist, &target.artist);

            // Reject if title is fundamentally different
            if title_sim < 0.3 {
                continue;
            }

            let mut total_score = (title_sim * 100.0) as i32 + (artist_sim * 60.0) as i32;
            total_score += duration_score;

            scored_candidates.push((song, total_score));
        }

        scored_candidates.sort_by(|a, b| b.1.cmp(&a.1));

        scored_candidates
            .first()
            .map(|(song, _)| *song)
            .ok_or_else(|| {
                format!(
                    "No suitable JioSaavn 320k candidate found for '{}' by '{}'",
                    target.title, target.artist
                )
            })
    }

    /// Extracts artist string following Softify specification rules.
    pub fn extract_artists(song: &JioSaavnSong) -> String {
        let mut artist_names = Vec::new();

        if let Some(more) = &song.more_info {
            if let Some(map) = &more.artist_map {
                for a in &map.primary_artists {
                    if let Some(name) = &a.name {
                        let trimmed = name.trim();
                        if !trimmed.is_empty() && !artist_names.contains(&trimmed.to_string()) {
                            artist_names.push(trimmed.to_string());
                        }
                    }
                }
                for a in &map.featured_artists {
                    if let Some(name) = &a.name {
                        let trimmed = name.trim();
                        if !trimmed.is_empty() && !artist_names.contains(&trimmed.to_string()) {
                            artist_names.push(trimmed.to_string());
                        }
                    }
                }
            }
        }

        // Fallback: If empty (common for Western/English songs), extract artist prefix from subtitle before " - "
        if artist_names.is_empty() {
            if let Some(sub) = &song.subtitle {
                if let Some((artist_prefix, _)) = sub.split_once(" - ") {
                    let cleaned = artist_prefix.trim();
                    if !cleaned.is_empty() {
                        return normalize_unicode_spaces(cleaned);
                    }
                } else if !sub.trim().is_empty() {
                    return normalize_unicode_spaces(sub.trim());
                }
            }
        }

        if artist_names.is_empty() {
            "Unknown Artist".to_string()
        } else {
            normalize_unicode_spaces(&artist_names.join(", "))
        }
    }

    pub fn parse_duration(val: &Option<serde_json::Value>) -> u32 {
        match val {
            Some(serde_json::Value::Number(num)) => num.as_u64().unwrap_or(0) as u32,
            Some(serde_json::Value::String(s)) => s.parse::<u32>().unwrap_or(0),
            _ => 0,
        }
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

fn decode_html_entities(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_artists_western_subtitle_fallback() {
        let song = JioSaavnSong {
            id: "123".to_string(),
            title: "Gym Rat".to_string(),
            subtitle: Some("Blackway, Black Caviar - Gym Rat".to_string()),
            duration: Some(serde_json::json!("180")),
            image: None,
            more_info: Some(JioSaavnMoreInfo {
                album: Some("Gym Rat".to_string()),
                encrypted_media_url: Some("encrypted_test".to_string()),
                artist_map: Some(JioSaavnArtistMap::default()),
            }),
        };

        let artist = JioSaavnResolver::extract_artists(&song);
        assert_eq!(artist, "Blackway, Black Caviar");
    }

    #[test]
    fn test_actor_disqualification() {
        let starring_song = JioSaavnSong {
            id: "456".to_string(),
            title: "Movie Song".to_string(),
            subtitle: Some("Actor - Movie".to_string()),
            duration: Some(serde_json::json!("200")),
            image: None,
            more_info: Some(JioSaavnMoreInfo {
                album: None,
                encrypted_media_url: Some("enc".to_string()),
                artist_map: Some(JioSaavnArtistMap {
                    primary_artists: vec![],
                    featured_artists: vec![],
                    artists: vec![JioSaavnArtist {
                        id: Some("1".to_string()),
                        name: Some("Famous Actor".to_string()),
                        role: Some("starring".to_string()),
                    }],
                }),
            }),
        };

        let target = TrackMetadata {
            id: "1".to_string(),
            title: "Movie Song".to_string(),
            artist: "Singer".to_string(),
            album: None,
            duration_seconds: 200,
            cover_url: None,
        };

        let candidates = vec![starring_song];
        let result =
            JioSaavnResolver::find_best_candidate(&candidates, &target, "Movie Song");
        assert!(result.is_err());
    }

    #[test]
    fn test_candidate_scoring_and_duration_bonus() {
        let song = JioSaavnSong {
            id: "789".to_string(),
            title: "Blinding Lights".to_string(),
            subtitle: Some("The Weeknd - After Hours".to_string()),
            duration: Some(serde_json::json!("200")),
            image: None,
            more_info: Some(JioSaavnMoreInfo {
                album: Some("After Hours".to_string()),
                encrypted_media_url: Some("valid_enc".to_string()),
                artist_map: Some(JioSaavnArtistMap {
                    primary_artists: vec![JioSaavnArtist {
                        id: Some("1".to_string()),
                        name: Some("The Weeknd".to_string()),
                        role: Some("primary_artists".to_string()),
                    }],
                    featured_artists: vec![],
                    artists: vec![],
                }),
            }),
        };

        let target = TrackMetadata {
            id: "t1".to_string(),
            title: "Blinding Lights".to_string(),
            artist: "The Weeknd".to_string(),
            album: None,
            duration_seconds: 203, // within 6s
            cover_url: None,
        };

        let candidates = vec![song];
        let best =
            JioSaavnResolver::find_best_candidate(&candidates, &target, "Blinding Lights The Weeknd")
                .expect("Should find best candidate");
        assert_eq!(best.id, "789");
    }
}
