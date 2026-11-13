pub mod hybrid;
pub mod jiosaavn;
pub mod lyrics;
pub mod spotify;
pub mod youtube;

use crate::models::{StreamResult, TrackMetadata};
use async_trait::async_trait;

#[async_trait]
pub trait StreamResolver: Send + Sync {
    async fn resolve(&self, track: &TrackMetadata) -> Result<StreamResult, String>;
}

#[async_trait]
impl StreamResolver for hybrid::HybridResolver {
    async fn resolve(&self, track: &TrackMetadata) -> Result<StreamResult, String> {
        self.resolve_stream(track).await
    }
}
