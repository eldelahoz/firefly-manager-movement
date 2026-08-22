//! CSV parsing and column mapping (pure application logic).
//!
//! Detects the delimiter, maps columns to domain fields, validates each row
//! (date + amount parsing), and returns valid movements alongside invalid
//! lines with their reason.

mod detect;
mod mapping;
mod parse;

pub use detect::detect_delimiter;
pub use mapping::{auto_detect_mapping, ColumnMapping};
pub use parse::{parse_csv, CsvParseResult, InvalidLine, ParsedMovement};
