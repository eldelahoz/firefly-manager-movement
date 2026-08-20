//! Errores de la capa de aplicación.
//!
//! `ApplicationError` es agnóstico a los detalles de infraestructura (no
//! referencia `diesel::result::Error` ni `reqwest::Error`): los adaptadores
//! de infraestructura mapean sus errores concretos a estas variantes.
//! Así se preserva la Dependency Rule (application no conoce los frameworks).

/// Error de un caso de uso.
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
