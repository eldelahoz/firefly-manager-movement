use async_trait::async_trait;

use crate::dtos::NewAppSettings;
use crate::error::ApplicationError;
use crate::ports::firefly::FireflyClient;
use ffm_domain::entities::AppSettings;

/// Persistence port for the single-row app settings (RF-19).
#[async_trait]
pub trait AppSettingsRepository: Send + Sync {
    async fn get(&self) -> Result<Option<AppSettings>, ApplicationError>;
    async fn save(&self, settings: NewAppSettings) -> Result<AppSettings, ApplicationError>;
}

/// Swaps the active FF3 connection at runtime (settings take precedence
/// over env vars, RF-19).
#[async_trait]
pub trait FireflyConnectionManager: Send + Sync {
    async fn reconfigure(&self, ff3_url: &str, ff3_token: &str) -> Result<(), ApplicationError>;
}

/// Builds a throwaway client for a candidate connection (verify action).
pub trait FireflyClientFactory: Send + Sync {
    fn build(&self, ff3_url: &str, ff3_token: &str) -> std::sync::Arc<dyn FireflyClient>;
}
