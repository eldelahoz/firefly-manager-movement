use std::sync::Arc;

use ffm_application::ports::{Ff3AccountRepository, FireflyClient};

#[derive(Clone)]
pub struct AppState {
    pub firefly: Arc<dyn FireflyClient>,
    pub ff3_accounts: Arc<dyn Ff3AccountRepository>,
    pub ff3_health_probe: bool,
}
