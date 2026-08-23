use std::sync::Arc;

use ffm_application::ports::{
    BatchRepository, CsvDecoder, CsvMappingRepository, Ff3AccountRepository,
    Ff3TransactionRepository, FireflyClient, MatchRepository, MovementRepository,
    ReconciliationRepository,
};

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
    pub ff3_health_probe: bool,
}
