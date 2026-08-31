//! CSV file decoder - detects encoding (BOM or heuristic) and decodes to UTF-8.

pub mod decoder;

pub use decoder::EncodingCsvDecoder;
