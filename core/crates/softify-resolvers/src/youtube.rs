use reqwest::Client;
use serde::{Deserialize, Serialize};
use softify_crypto::{
    clean_title, is_disqualified_variant, normalize_unicode_spaces, score_duration_proximity,
    token_similarity,
};
use softify_models::{StreamResult, TrackMetadata};

const YTM_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";
const YTM_SEARCH_PARAMS: &str = "EgWKAQIIAWoQEAMQBBAJEAoQBRAREBAQFQ%3D%3D";

#[derive(Clone, Debug)]
pub struct YouTubeMusicResolver {
    client: Client,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YtCandidate {
    pub video_id: String,
    pub title: String,
    pub author: String,
    pub duration_seconds: u32,
    pub is_topic: bool,
}

impl YouTubeMusicResolver {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(YTM_USER_AGENT)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    /// Searches YouTube Music using InnerTube WEB_REMIX client.
    pub async fn search_candidates(&self, query: &str) -> Result<Vec<YtCandidate>, String> {
        let body = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20240422.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "query": query,
            "params": YTM_SEARCH_PARAMS
        });

        let resp = self
            .client
            .post("https://music.youtube.com/youtubei/v1/search?prettyPrint=false")
            .header("Content-Type", "application/json")
            .header("User-Agent", YTM_USER_AGENT)
            .header("Referer", "https://music.youtube.com/")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("YouTube Music search request failed: {e}"))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read YouTube Music response: {e}"))?;

        Self::parse_search_candidates(&text)
    }

    /// Resolves stream for a given track using InnerTube fallback with Topic channel prioritization.
    pub async fn resolve_stream(&self, track: &TrackMetadata) -> Result<StreamResult, String> {
        let query = format!("{} {}", track.title, track.artist);
        let candidates = self.search_candidates(&query).await?;

        let best = Self::score_and_select_candidate(&candidates, track, &query)?;

        // Resolve stream URL using Android client player API
        self.resolve_video_stream(&best.video_id).await
    }

    /// Directly extracts audio stream URL from any YouTube video URL or ID.
    pub async fn extract_video_audio(&self, url_or_id: &str) -> Result<StreamResult, String> {
        let video_id = Self::extract_youtube_id(url_or_id).ok_or_else(|| {
            format!("Failed to extract valid 11-character YouTube video ID from: '{url_or_id}'")
        })?;

        let mut stream = self.resolve_video_stream(&video_id).await?;
        stream.provider = "youtube_direct".to_string();
        Ok(stream)
    }

    /// Calls InnerTube player endpoint to extract direct audio playback URL.
    pub async fn resolve_video_stream(&self, video_id: &str) -> Result<StreamResult, String> {
        let body = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "ANDROID_MUSIC",
                    "clientVersion": "6.42.52",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": video_id
        });

        let resp = self
            .client
            .post("https://music.youtube.com/youtubei/v1/player")
            .header("Content-Type", "application/json")
            .header("User-Agent", YTM_USER_AGENT)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("YouTube player request failed: {e}"))?;

        let root: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse player JSON: {e}"))?;

        Self::parse_player_stream_result(&root)
    }

    /// Parses player response JSON for the highest bitrate audio format.
    pub fn parse_player_stream_result(root: &serde_json::Value) -> Result<StreamResult, String> {
        let formats = root
            .pointer("/streamingData/adaptiveFormats")
            .and_then(|f| f.as_array())
            .ok_or_else(|| "No adaptiveFormats found in player response".to_string())?;

        // Filter for audio streams with direct URLs
        let mut audio_streams: Vec<(&serde_json::Value, u64)> = Vec::new();
        for f in formats {
            let mime = f.get("mimeType").and_then(|m| m.as_str()).unwrap_or("");
            if mime.starts_with("audio/") {
                let bitrate = f.get("bitrate").and_then(|b| b.as_u64()).unwrap_or(0);
                if f.get("url").and_then(|u| u.as_str()).is_some() {
                    audio_streams.push((f, bitrate));
                }
            }
        }

        audio_streams.sort_by(|a, b| b.1.cmp(&a.1));

        let (best_format, bitrate) = audio_streams
            .first()
            .ok_or_else(|| "No direct audio stream URL found in player formats".to_string())?;

        let url = best_format
            .get("url")
            .and_then(|u| u.as_str())
            .unwrap()
            .to_string();

        let mime = best_format
            .get("mimeType")
            .and_then(|m| m.as_str())
            .unwrap_or("audio/mp4");

        let (container, codec) = if mime.contains("webm") {
            ("webm".to_string(), "opus".to_string())
        } else {
            ("m4a".to_string(), "aac".to_string())
        };

        Ok(StreamResult {
            stream_url: url,
            bitrate: *bitrate as u32,
            container,
            codec,
            provider: "youtube_innertube".to_string(),
        })
    }

    /// Extracts standard 11-character YouTube video ID from various URL formats or raw ID.
    pub fn extract_youtube_id(input: &str) -> Option<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        // If it's already an 11-char ID without slashes or question marks
        if trimmed.len() == 11
            && trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Some(trimmed.to_string());
        }

        // Handle youtu.be/ID
        if let Some(idx) = trimmed.find("youtu.be/") {
            let after = &trimmed[idx + "youtu.be/".len()..];
            let id = after.split(['?', '&', '/', '#']).next().unwrap_or("");
            if id.len() == 11 {
                return Some(id.to_string());
            }
        }

        // Handle /watch?v=ID or /v/ID or /embed/ID
        if let Some(idx) = trimmed.find("v=") {
            let after = &trimmed[idx + 2..];
            let id = after.split(['&', '#', '/']).next().unwrap_or("");
            if id.len() == 11 {
                return Some(id.to_string());
            }
        }

        if let Some(idx) = trimmed.find("/embed/") {
            let after = &trimmed[idx + "/embed/".len()..];
            let id = after.split(['?', '&', '/', '#']).next().unwrap_or("");
            if id.len() == 11 {
                return Some(id.to_string());
            }
        }

        if let Some(idx) = trimmed.find("/v/") {
            let after = &trimmed[idx + "/v/".len()..];
            let id = after.split(['?', '&', '/', '#']).next().unwrap_or("");
            if id.len() == 11 {
                return Some(id.to_string());
            }
        }

        None
    }

    /// Selects best candidate with Topic Channel (+80 points) prioritization.
    pub fn score_and_select_candidate<'a>(
        candidates: &'a [YtCandidate],
        target: &TrackMetadata,
        user_query: &str,
    ) -> Result<&'a YtCandidate, String> {
        let mut scored: Vec<(&'a YtCandidate, i32)> = Vec::new();

        for candidate in candidates {
            // Variant disqualification
            if is_disqualified_variant(&candidate.title, user_query) {
                continue;
            }

            // Duration proximity check
            let duration_score = if target.duration_seconds > 0 && candidate.duration_seconds > 0 {
                match score_duration_proximity(candidate.duration_seconds, target.duration_seconds)
                {
                    Some(score) => score,
                    None => continue, // Disqualified: delta > 90s
                }
            } else {
                0
            };

            let cleaned_target_title = clean_title(&target.title);
            let cleaned_cand_title = clean_title(&candidate.title);

            let title_sim = token_similarity(&cleaned_cand_title, &cleaned_target_title);
            let author_sim = token_similarity(&candidate.author, &target.artist);

            if title_sim < 0.25 {
                continue;
            }

            let mut total_score = (title_sim * 100.0) as i32 + (author_sim * 50.0) as i32;
            total_score += duration_score;

            // Topic Channel Prioritization Law: +80 confidence points for official record-label uploads
            if candidate.is_topic {
                total_score += 80;
            }

            scored.push((candidate, total_score));
        }

        scored.sort_by(|a, b| b.1.cmp(&a.1));

        scored
            .first()
            .map(|(c, _)| *c)
            .ok_or_else(|| format!("No suitable YouTube Music candidate found for '{}'", target.title))
    }

    /// Parses search candidate JSON items from InnerTube response.
    pub fn parse_search_candidates(json_str: &str) -> Result<Vec<YtCandidate>, String> {
        let root: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse InnerTube JSON: {e}"))?;

        let mut candidates = Vec::new();
        Self::extract_renderers(&root, &mut candidates);
        Ok(candidates)
    }

    fn extract_renderers(val: &serde_json::Value, out: &mut Vec<YtCandidate>) {
        if let Some(obj) = val.as_object() {
            if let Some(renderer) = obj.get("musicResponsiveListItemRenderer") {
                if let Some(candidate) = Self::parse_single_renderer(renderer) {
                    out.push(candidate);
                }
            } else {
                for (_, v) in obj {
                    Self::extract_renderers(v, out);
                }
            }
        } else if let Some(arr) = val.as_array() {
            for v in arr {
                Self::extract_renderers(v, out);
            }
        }
    }

    fn parse_single_renderer(renderer: &serde_json::Value) -> Option<YtCandidate> {
        let video_id = renderer
            .pointer("/playlistItemData/videoId")
            .or_else(|| {
                renderer
                    .pointer("/doubleTapCommand/watchEndpoint/videoId")
                    .or_else(|| renderer.pointer("/onTap/watchEndpoint/videoId"))
            })
            .and_then(|v| v.as_str())?
            .to_string();

        let flex_columns = renderer.get("flexColumns")?.as_array()?;

        // Column 0: Title
        let title_runs = flex_columns
            .first()?
            .pointer("/musicResponsiveListItemFlexColumnRenderer/text/runs")?
            .as_array()?;
        let raw_title = title_runs.first()?.get("text")?.as_str()?;
        let title = normalize_unicode_spaces(raw_title);

        // Column 1: Subtitle containing author and duration
        let subtitle_runs = flex_columns
            .get(1)?
            .pointer("/musicResponsiveListItemFlexColumnRenderer/text/runs")?
            .as_array()?;

        let mut author = "Unknown Artist".to_string();
        let mut duration_seconds = 0;
        let mut is_topic = false;

        for run in subtitle_runs {
            if let Some(text) = run.get("text").and_then(|t| t.as_str()) {
                let trimmed = text.trim();
                if trimmed.contains(':') && trimmed.chars().all(|c| c.is_ascii_digit() || c == ':')
                {
                    duration_seconds = parse_yt_duration(trimmed);
                } else if !trimmed.is_empty() && trimmed != "•" && author == "Unknown Artist" {
                    author = normalize_unicode_spaces(trimmed);
                    if author.ends_with("- Topic") {
                        is_topic = true;
                    }
                }
            }
        }

        Some(YtCandidate {
            video_id,
            title,
            author,
            duration_seconds,
            is_topic,
        })
    }
}

fn parse_yt_duration(s: &str) -> u32 {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        2 => {
            let m: u32 = parts[0].parse().unwrap_or(0);
            let s: u32 = parts[1].parse().unwrap_or(0);
            m * 60 + s
        }
        3 => {
            let h: u32 = parts[0].parse().unwrap_or(0);
            let m: u32 = parts[1].parse().unwrap_or(0);
            let s: u32 = parts[2].parse().unwrap_or(0);
            h * 3600 + m * 60 + s
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_youtube_id() {
        // Full YouTube URL
        assert_eq!(
            YouTubeMusicResolver::extract_youtube_id(
                "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
            ),
            Some("dQw4w9WgXcQ".to_string())
        );

        // YouTube Music URL with extra params
        assert_eq!(
            YouTubeMusicResolver::extract_youtube_id(
                "https://music.youtube.com/watch?v=dQw4w9WgXcQ&feature=share"
            ),
            Some("dQw4w9WgXcQ".to_string())
        );

        // Short youtu.be URL
        assert_eq!(
            YouTubeMusicResolver::extract_youtube_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );

        // Embed URL
        assert_eq!(
            YouTubeMusicResolver::extract_youtube_id("https://www.youtube.com/embed/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );

        // /v/ URL
        assert_eq!(
            YouTubeMusicResolver::extract_youtube_id("https://www.youtube.com/v/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );

        // Raw 11-char ID
        assert_eq!(
            YouTubeMusicResolver::extract_youtube_id("dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );

        // Invalid inputs
        assert_eq!(YouTubeMusicResolver::extract_youtube_id(""), None);
        assert_eq!(
            YouTubeMusicResolver::extract_youtube_id("https://spotify.com"),
            None
        );
    }

    #[test]
    fn test_topic_channel_prioritization_and_scoring() {
        let target = TrackMetadata {
            id: "t1".to_string(),
            title: "Blinding Lights".to_string(),
            artist: "The Weeknd".to_string(),
            album: None,
            duration_seconds: 200,
            cover_url: None,
        };

        let user_upload = YtCandidate {
            video_id: "user_vid".to_string(),
            title: "Blinding Lights".to_string(),
            author: "Some User Uploads".to_string(),
            duration_seconds: 200,
            is_topic: false,
        };

        let topic_upload = YtCandidate {
            video_id: "topic_vid".to_string(),
            title: "Blinding Lights".to_string(),
            author: "The Weeknd - Topic".to_string(),
            duration_seconds: 200,
            is_topic: true,
        };

        let candidates = vec![user_upload, topic_upload];
        let best = YouTubeMusicResolver::score_and_select_candidate(
            &candidates,
            &target,
            "Blinding Lights The Weeknd",
        )
        .expect("Topic candidate should be selected");

        assert_eq!(best.video_id, "topic_vid");
        assert!(best.is_topic);
    }

    #[test]
    fn test_parse_player_stream_result_highest_bitrate() {
        let player_json = serde_json::json!({
            "streamingData": {
                "adaptiveFormats": [
                    {
                        "mimeType": "video/mp4; codecs=\"avc1.4d401f\"",
                        "bitrate": 1500000,
                        "url": "https://googlevideo.com/videoplayback_video"
                    },
                    {
                        "mimeType": "audio/webm; codecs=\"opus\"",
                        "bitrate": 160000,
                        "url": "https://googlevideo.com/videoplayback_opus160"
                    },
                    {
                        "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"",
                        "bitrate": 128000,
                        "url": "https://googlevideo.com/videoplayback_aac128"
                    }
                ]
            }
        });

        let stream = YouTubeMusicResolver::parse_player_stream_result(&player_json).unwrap();
        assert_eq!(stream.bitrate, 160000);
        assert_eq!(stream.container, "webm");
        assert_eq!(stream.codec, "opus");
        assert_eq!(
            stream.stream_url,
            "https://googlevideo.com/videoplayback_opus160"
        );
        assert_eq!(stream.provider, "youtube_innertube");
    }
}
