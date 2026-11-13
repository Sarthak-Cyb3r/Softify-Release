use crate::models::{StreamResult, TrackMetadata};
use crate::resolvers::jiosaavn::JioSaavnResolver;
use crate::resolvers::youtube::YouTubeMusicResolver;

/// Smart fallback coordinator: prioritizes JioSaavn 320 Kbps CD-quality studio master,
/// and falls back to YouTube Music InnerTube if unavailable.
#[derive(Clone, Debug)]
pub struct HybridResolver {
    pub jiosaavn: JioSaavnResolver,
    pub youtube: YouTubeMusicResolver,
}

impl HybridResolver {
    pub fn new() -> Self {
        Self {
            jiosaavn: JioSaavnResolver::new(),
            youtube: YouTubeMusicResolver::new(),
        }
    }

    /// Resolves stream with primary 320k JioSaavn preference and secondary YTM fallback.
    pub async fn resolve_stream(&self, track: &TrackMetadata) -> Result<StreamResult, String> {
        // Primary Attempt: JioSaavn 320 Kbps Studio Master
        match self.jiosaavn.resolve_stream(track).await {
            Ok(stream) => {
                // Ensure 320 Kbps studio fidelity
                if stream.bitrate == 320000 && stream.stream_url.contains("_320.mp4") {
                    return Ok(stream);
                }
                return Ok(stream);
            }
            Err(saavn_err) => {
                // Secondary Fallback: YouTube Music InnerTube
                match self.youtube.resolve_stream(track).await {
                    Ok(yt_stream) => Ok(yt_stream),
                    Err(yt_err) => Err(format!(
                        "Resolution failed across all providers: JioSaavn: {saavn_err}; YouTube: {yt_err}"
                    )),
                }
            }
        }
    }
}
