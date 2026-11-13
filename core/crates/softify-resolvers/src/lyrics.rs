use reqwest::Client;
use serde::{Deserialize, Serialize};
use softify_models::Lyrics;

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
    ) -> Result<Option<Lyrics>, String> {
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

        if data.synced_lyrics.is_some() || data.plain_lyrics.is_some() {
            Ok(Some(Lyrics {
                plain_lyrics: data.plain_lyrics,
                synced_lyrics: data.synced_lyrics,
                is_romanized: false,
            }))
        } else {
            Ok(None)
        }
    }

    /// Fetches plain lyrics from JioSaavn if available.
    pub async fn fetch_jiosaavn(&self, lyrics_id: &str) -> Result<Option<Lyrics>, String> {
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
            return Ok(Some(Lyrics {
                plain_lyrics: Some(cleaned),
                synced_lyrics: None,
                is_romanized: false,
            }));
        }

        Ok(None)
    }

    /// Resolves lyrics by querying LRCLIB first, with optional romanization support.
    pub async fn resolve_lyrics(
        &self,
        title: &str,
        artist: &str,
        duration_seconds: Option<u32>,
    ) -> Option<Lyrics> {
        if let Ok(Some(lyrics)) = self.fetch_lrclib(title, artist, duration_seconds).await {
            return Some(lyrics);
        }
        None
    }
}

/// Helper that romanizes lyrics, converting non-Latin scripts (e.g. Devanagari) to Latin phonetics
/// while strictly preserving synchronized timestamp tags (e.g. `[00:15.20]`).
pub fn romanize_lyrics(lyrics: &Lyrics) -> Lyrics {
    let plain_romanized = lyrics.plain_lyrics.as_deref().map(romanize_text);
    let synced_romanized = lyrics.synced_lyrics.as_deref().map(romanize_synced_text);

    Lyrics {
        plain_lyrics: plain_romanized,
        synced_lyrics: synced_romanized,
        is_romanized: true,
    }
}

/// Checks if string contains non-Latin scripts (Devanagari, Indic, CJK, etc.)
pub fn is_non_latin(text: &str) -> bool {
    text.chars().any(|c| {
        let code = c as u32;
        // Devanagari (0x0900..=0x097F), CJK Unified Ideographs, Hangul, Cyrillic
        (0x0900..=0x097F).contains(&code) || (0x4E00..=0x9FFF).contains(&code)
    })
}

/// Romanizes plain lyrics text line by line.
pub fn romanize_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&romanize_line(line));
    }
    out
}

/// Romanizes synchronized lyrics while keeping timecodes (e.g. `[01:23.45]`) verbatim.
pub fn romanize_synced_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        if !out.is_empty() {
            out.push('\n');
        }

        // Check if line starts with timestamp like [00:15.20]
        if line.starts_with('[') {
            if let Some(close_idx) = line.find(']') {
                let (timestamp, rest) = line.split_at(close_idx + 1);
                out.push_str(timestamp);
                out.push_str(&romanize_line(rest));
                continue;
            }
        }
        out.push_str(&romanize_line(line));
    }
    out
}

/// Converts Devanagari syllables and characters in a line to standard Latin phonetics.
fn romanize_line(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::with_capacity(line.len() * 2);
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let code = c as u32;

        if (0x0900..=0x097F).contains(&code) {
            // Check if it's an independent vowel
            let vowel = match c {
                'अ' => Some("a"),
                'आ' => Some("aa"),
                'इ' => Some("i"),
                'ई' => Some("ee"),
                'उ' => Some("u"),
                'ऊ' => Some("oo"),
                'ऋ' => Some("ri"),
                'ए' => Some("e"),
                'ऐ' => Some("ai"),
                'ओ' => Some("o"),
                'औ' => Some("au"),
                'अं' => Some("am"),
                'अः' => Some("ah"),
                _ => None,
            };

            if let Some(v) = vowel {
                result.push_str(v);
                i += 1;
                continue;
            }

            // Check if it's a consonant
            let consonant = match c {
                'क' => Some("k"),
                'ख' => Some("kh"),
                'ग' => Some("g"),
                'घ' => Some("gh"),
                'ङ' => Some("ng"),
                'च' => Some("ch"),
                'छ' => Some("chh"),
                'ज' => Some("j"),
                'झ' => Some("jh"),
                'ञ' => Some("ny"),
                'ट' => Some("t"),
                'ठ' => Some("th"),
                'ड' => Some("d"),
                'ढ' => Some("dh"),
                'ण' => Some("n"),
                'त' => Some("t"),
                'थ' => Some("th"),
                'द' => Some("d"),
                'ध' => Some("dh"),
                'न' => Some("n"),
                'प' => Some("p"),
                'फ' => Some("ph"),
                'ब' => Some("b"),
                'भ' => Some("bh"),
                'म' => Some("m"),
                'य' => Some("y"),
                'र' => Some("r"),
                'ल' => Some("l"),
                'व' => Some("v"),
                'श' => Some("sh"),
                'ष' => Some("sh"),
                'स' => Some("s"),
                'ह' => Some("h"),
                _ => None,
            };

            if let Some(cons) = consonant {
                result.push_str(cons);
                // Lookahead for matra or halant (virama)
                if i + 1 < chars.len() {
                    let next_c = chars[i + 1];
                    let matra = match next_c {
                        'ा' => Some("aa"),
                        'ि' => Some("i"),
                        'ी' => Some("ee"),
                        'ु' => Some("u"),
                        'ू' => Some("oo"),
                        'ृ' => Some("ri"),
                        'े' => Some("e"),
                        'ै' => Some("ai"),
                        'ो' => Some("o"),
                        'ौ' => Some("au"),
                        'ं' => Some("m"),
                        'ँ' => Some("n"),
                        '्' => Some(""), // virama / halant: suppress inherent vowel
                        _ => None,
                    };

                    if let Some(m) = matra {
                        result.push_str(m);
                        i += 2;
                        continue;
                    }
                }
                // Default inherent vowel 'a' if not suppressed or modified
                result.push('a');
                i += 1;
                continue;
            }

            // Standalone matras or signs
            match c {
                'ा' => result.push_str("aa"),
                'ि' => result.push('i'),
                'ी' => result.push_str("ee"),
                'ु' => result.push('u'),
                'ू' => result.push_str("oo"),
                'े' => result.push('e'),
                'ै' => result.push_str("ai"),
                'ो' => result.push('o'),
                'ौ' => result.push_str("au"),
                'ं' => result.push('m'),
                'ँ' => result.push('n'),
                '्' => {}
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
        i += 1;
    }

    result
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
    fn test_parse_lrclib_response() {
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
        assert_eq!(
            parsed.plain_lyrics.as_deref(),
            Some("Yeah, I been tryna call")
        );
    }

    #[test]
    fn test_romanize_devanagari_and_preserve_timestamps() {
        let synced = "[00:15.20] तुम ही हो\n[00:20.10] अब तुम ही हो";
        let romanized = romanize_synced_text(synced);

        assert!(romanized.contains("[00:15.20]"));
        assert!(romanized.contains("[00:20.10]"));
        assert!(romanized.to_lowercase().contains("tum"));
        assert!(romanized.to_lowercase().contains("hee"));
        assert!(romanized.to_lowercase().contains("ho"));
    }

    #[test]
    fn test_romanize_lyrics_struct() {
        let original = Lyrics {
            plain_lyrics: Some("तुम ही हो".to_string()),
            synced_lyrics: Some("[00:05.00] तुम ही हो".to_string()),
            is_romanized: false,
        };

        let processed = romanize_lyrics(&original);
        assert!(processed.is_romanized);
        assert!(processed.plain_lyrics.unwrap().to_lowercase().contains("tum"));
        assert!(processed.synced_lyrics.unwrap().starts_with("[00:05.00]"));
    }
}
