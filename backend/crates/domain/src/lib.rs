//! ffm-domain — entidades y value objects del dominio.
//! Única dependencia externa permitida: `chrono` (tipo de dato para fechas).
//! (Dependency Rule: nada de frameworks/serde/diesel entra aquí.)

pub mod entities;
pub mod ids;
pub mod matching_engine;
pub mod value_objects;
