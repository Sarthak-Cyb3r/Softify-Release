use crate::models::{StreamResult, TrackMetadata};
use crate::normalizer::{
    clean_title, is_disqualified_variant, normalize_unicode_spaces, score_duration_proximity,
    token_similarity,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

        let formats = root
            .pointer("/streamingData/adaptiveFormats")
            .and_then(|f| f.as_array())
            .ok_or_else(|| "No adaptiveFormats found in player response".to_string())?;

        // Filter for audio streams
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

        // Search recursively for musicResponsiveListItemRenderer
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
        // Video ID
        let video_id = renderer
            .pointer("/playlistItemData/videoId")
            .or_else(|| {
                renderer.pointer("/navigationEndpoint/watchEndpoint/videoId")
            })
            .or_else(|| {
                renderer.pointer("/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId")
            })
            .and_then(|v| v.as_str())?
            .to_string();

        // Title from flexColumns[0]
        let title_runs = renderer
            .pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs")
            .and_then(|r| r.as_array())?;
        let title = title_runs
            .first()
            .and_then(|r| r.get("text"))
            .and_then(|t| t.as_str())?;

        // Author from flexColumns[1]
        let mut author = "Unknown Artist".to_string();
        let mut is_topic = false;
        let mut duration_seconds = 0;

        if let Some(col1_runs) = renderer
            .pointer("/flexColumns/1/musicResponsiveListItemFlexColumnRenderer/text/runs")
            .and_then(|r| r.as_array())
        {
            for run in col1_runs {
                if let Some(text) = run.get("text").and_then(|t| t.as_str()) {
                    let trimmed = text.trim();
                    // Topic channel detection
                    if trimmed.ends_with("- Topic") {
                        is_topic = true;
                        author = trimmed.to_string();
                    } else if author == "Unknown Artist" && !trimmed.is_empty() && trimmed != "•" {
                        author = trimmed.to_string();
                    }

                    // Duration detection (e.g. "3:20" or "03:20")
                    if trimmed.contains(':') && trimmed.len() <= 8 {
                        duration_seconds = parse_time_to_seconds(trimmed);
                    }
                }
            }
        }

        Some(YtCandidate {
            video_id,
            title: normalize_unicode_spaces(title),
            author: normalize_unicode_spaces(&author),
            duration_seconds,
            is_topic,
        })
    }
}

fn parse_time_to_seconds(time_str: &str) -> u32 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        let mins: u32 = parts[0].parse().unwrap_or(0);
        let secs: u32 = parts[1].parse().unwrap_or(0);
        mins * 60 + secs
    } else if parts.len() == 3 {
        let hrs: u32 = parts[0].parse().unwrap_or(0);
        let mins: u32 = parts[1].parse().unwrap_or(0);
        let secs: u32 = parts[2].parse().unwrap_or(0);
        hrs * 3600 + mins * 60 + secs
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_channel_prioritization() {
        let normal_candidate = YtCandidate {
            video_id: "abc".to_string(),
            title: "Blinding Lights".to_string(),
            author: "Random Uploader".to_string(),
            duration_seconds: 200,
            is_topic: false,
        };

        let topic_candidate = YtCandidate {
            video_id: "def".to_string(),
            title: "Blinding Lights".to_string(),
            author: "The Weeknd - Topic".to_string(),
            duration_seconds: 200,
            is_topic: true,
        };

        let target = TrackMetadata {
            id: "1".to_string(),
            title: "Blinding Lights".to_string(),
            artist: "The Weeknd".to_string(),
            album: None,
            duration_seconds: 200,
            cover_url: None,
        };

        let candidates = vec![normal_candidate, topic_candidate];
        let best = YouTubeMusicResolver::score_and_select_candidate(
            &candidates,
            &target,
            "Blinding Lights",
        )
        .expect("Should select best candidate");

        // Must select the official - Topic channel candidate (+80 points)
        assert_eq!(best.video_id, "def");
        assert!(best.is_topic);
    }
}
