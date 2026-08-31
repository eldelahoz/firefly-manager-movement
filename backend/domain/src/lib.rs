//! ffm-domain - domain entities and value objects.
//! Only allowed external dependency: `chrono` (data type for dates).
//! (Dependency Rule: no frameworks/serde/diesel enter here.)

pub mod entities;
pub mod ids;
pub mod matching_engine;
pub mod value_objects;
