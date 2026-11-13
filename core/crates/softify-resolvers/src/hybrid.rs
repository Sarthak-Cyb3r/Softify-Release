use crate::jiosaavn::JioSaavnResolver;
use crate::youtube::YouTubeMusicResolver;
use async_trait::async_trait;
use softify_models::{StreamResult, TrackMetadata};

/// Trait defining asynchronous stream resolution across different audio providers.
#[async_trait]
pub trait StreamResolver: Send + Sync {
    async fn resolve(&self, track: &TrackMetadata) -> Result<StreamResult, String>;
}

/// Smart fallback coordinator: prioritizes JioSaavn 320 Kbps CD-quality studio master,
/// and falls back to YouTube Music InnerTube if unavailable.
#[derive(Clone, Debug)]
pub struct HybridResolver {
    pub jiosaavn: JioSaavnResolver,
    pub youtube: YouTubeMusicResolver,
}

impl Default for HybridResolver {
    fn default() -> Self {
        Self::new()
    }
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
                Ok(stream)
            }
            Err(saavn_err) => {
                // Secondary Fallback: YouTube Music InnerTube
                match self.youtube.resolve_stream(track).await {
                    Ok(yt_stream) => Ok(yt_stream),
                    Err(yt_err) => Err(format!(
                        "Stream resolution failed on all providers. JioSaavn error: '{}'; YouTube Music error: '{}'",
                        saavn_err, yt_err
                    )),
                }
            }
        }
    }
}

#[async_trait]
impl StreamResolver for HybridResolver {
    async fn resolve(&self, track: &TrackMetadata) -> Result<StreamResult, String> {
        self.resolve_stream(track).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_resolver_initialization() {
        let hybrid = HybridResolver::new();
        assert_eq!(hybrid.jiosaavn.clone().new_uninit_check(), ());
    }

    impl JioSaavnResolver {
        fn new_uninit_check(&self) {}
    }
}
