use std::sync::Arc;

use ffm_application::ports::{
    AppSettingsRepository, BatchRepository, CreationRepository, CsvDecoder, CsvMappingRepository,
    DeletedAuditRepository, Ff3AccountRepository, Ff3TransactionRepository, FireflyClient,
    FireflyClientFactory, FireflyConnectionManager, MatchRepository, MovementRepository,
    ReconciliationRepository,
};
use ffm_application::use_cases::app_settings::EnvDefaults;

#[derive(Clone)]
pub struct AppState {
    pub firefly: Arc<dyn FireflyClient>,
    pub ff3_accounts: Arc<dyn Ff3AccountRepository>,
    pub csv_decoder: Arc<dyn CsvDecoder>,
    pub batches: Arc<dyn BatchRepository>,
    pub movements: Arc<dyn MovementRepository>,
    pub csv_mappings: Arc<dyn CsvMappingRepository>,
    pub reconciliations: Arc<dyn ReconciliationRepository>,
    pub matches: Arc<dyn MatchRepository>,
    pub ff3_transactions: Arc<dyn Ff3TransactionRepository>,
    pub creations: Arc<dyn CreationRepository>,
    pub deleted_audits: Arc<dyn DeletedAuditRepository>,
    pub ff3_health_probe: bool,
    pub app_settings: Arc<dyn AppSettingsRepository>,
    pub firefly_manager: Arc<dyn FireflyConnectionManager>,
    pub firefly_factory: Arc<dyn FireflyClientFactory>,
    pub env_defaults: EnvDefaults,
}
