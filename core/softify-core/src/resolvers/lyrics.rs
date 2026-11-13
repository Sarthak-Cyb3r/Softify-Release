use reqwest::Client;
use serde::{Deserialize, Serialize};

const LYRICS_USER_AGENT: &str =
    "Softify-NextGen/1.0 (https://github.com/Sarthak-Cyb3r/Softify-Release)";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LrclibResponse {
    pub id: Option<u64>,
    #[serde(rename = "trackName")]
    pub track_name: Option<String>,
    #[serde(rename = "artistName")]
    pub artist_name: Option<String>,
    #[serde(rename = "plainLyrics")]
    pub plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    pub synced_lyrics: Option<String>,
    #[serde(default)]
    pub instrumental: bool,
}

#[derive(Clone, Debug)]
pub struct LyricsResolver {
    client: Client,
}

impl LyricsResolver {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(LYRICS_USER_AGENT)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    /// Fetches synchronized and plain lyrics from LRCLIB.
    pub async fn fetch_lrclib(
        &self,
        title: &str,
        artist: &str,
        duration_seconds: Option<u32>,
    ) -> Result<Option<String>, String> {
        let mut url = format!(
            "https://lrclib.net/api/get?track_name={}&artist_name={}",
            urlencoding_encode(title),
            urlencoding_encode(artist)
        );

        if let Some(dur) = duration_seconds {
            if dur > 0 {
                url.push_str(&format!("&duration={}", dur));
            }
        }

        let resp = self
            .client
            .get(&url)
            .header("User-Agent", LYRICS_USER_AGENT)
            .send()
            .await
            .map_err(|e| format!("LRCLIB HTTP request failed: {e}"))?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let data: LrclibResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse LRCLIB response: {e}"))?;

        // Prioritize synced lyrics (with [mm:ss.xx] timestamps), fallback to plain lyrics
        if let Some(synced) = data.synced_lyrics {
            if !synced.trim().is_empty() {
                return Ok(Some(synced));
            }
        }

        if let Some(plain) = data.plain_lyrics {
            if !plain.trim().is_empty() {
                return Ok(Some(plain));
            }
        }

        Ok(None)
    }

    /// Fetches plain lyrics from JioSaavn if available.
    pub async fn fetch_jiosaavn(&self, lyrics_id: &str) -> Result<Option<String>, String> {
        let url = format!(
            "https://www.jiosaavn.com/api.php?__call=lyrics.getLyrics&_format=json&ctx=web6dot0&api_version=4&lyrics_id={}",
            urlencoding_encode(lyrics_id)
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("JioSaavn lyrics request failed: {e}"))?;

        let text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read JioSaavn lyrics body: {e}"))?;

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse JioSaavn lyrics JSON: {e}"))?;

        if let Some(lyrics_raw) = parsed.get("lyrics").and_then(|l| l.as_str()) {
            let cleaned = lyrics_raw
                .replace("<br>", "\n")
                .replace("<br/>", "\n")
                .replace("<br />", "\n");
            return Ok(Some(cleaned));
        }

        Ok(None)
    }

    /// Resolves lyrics by querying LRCLIB first, then JioSaavn if lyrics_id is known.
    pub async fn resolve_lyrics(
        &self,
        title: &str,
        artist: &str,
        duration_seconds: Option<u32>,
    ) -> Option<String> {
        // Query LRCLIB primary
        if let Ok(Some(lyrics)) = self.fetch_lrclib(title, artist, duration_seconds).await {
            return Some(lyrics);
        }

        None
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
    fn test_parse_lrclib_synced_priority() {
        let json = r#"{
            "id": 100,
            "trackName": "Blinding Lights",
            "artistName": "The Weeknd",
            "plainLyrics": "Yeah, I been tryna call",
            "syncedLyrics": "[00:15.20] Yeah, I been tryna call\n[00:18.40] I been on my own"
        }"#;

        let parsed: LrclibResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed.synced_lyrics.as_deref(),
            Some("[00:15.20] Yeah, I been tryna call\n[00:18.40] I been on my own")
        );
    }
}
