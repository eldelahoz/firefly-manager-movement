//! Application layer errors.
//!
//! `ApplicationError` is agnostic to infrastructure details (it does not
//! reference `diesel::result::Error` or `reqwest::Error`): infrastructure
//! adapters map their concrete errors to these variants.
//! This preserves the Dependency Rule (application knows no frameworks).

/// A use case error.
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("recurso no encontrado: {0}")]
    NotFound(String),

    #[error("error de persistencia: {0}")]
    Database(String),

    #[error("error del cliente Firefly III: {0}")]
    FireflyClient(String),

    #[error("error de parseo: {0}")]
    Parse(String),

    #[error("estado inválido: {0}")]
    InvalidState(String),

    #[error("conflicto: {0}")]
    Conflict(String),

    #[error("error de configuración: {0}")]
    Config(String),

    #[error("error de I/O: {0}")]
    Io(String),
}
