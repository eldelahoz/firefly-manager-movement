//! `EncodingCsvDecoder` - decodes raw bytes to a UTF-8 string.
//!
//! Detection strategy:
//! 1. BOM present → decode with the BOM's encoding (UTF-8, UTF-16LE, UTF-16BE).
//! 2. No BOM → try UTF-8; if invalid bytes, fall back to windows-1252 (common
//!    for European bank CSVs).

use async_trait::async_trait;
use encoding_rs::{UTF_16BE, UTF_16LE, UTF_8, WINDOWS_1252};

use ffm_application::error::ApplicationError;
use ffm_application::ports::CsvDecoder;

pub struct EncodingCsvDecoder;

#[async_trait]
impl CsvDecoder for EncodingCsvDecoder {
    async fn decode(&self, bytes: Vec<u8>) -> Result<String, ApplicationError> {
        Ok(decode_bytes(&bytes))
    }
}

fn decode_bytes(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let (s, _) = UTF_8.decode_with_bom_removal(bytes);
        return s.into_owned();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let (s, _) = UTF_16LE.decode_with_bom_removal(bytes);
        return s.into_owned();
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let (s, _) = UTF_16BE.decode_with_bom_removal(bytes);
        return s.into_owned();
    }

    let (s, had_errors) = UTF_8.decode_without_bom_handling(bytes);
    if !had_errors {
        return s.into_owned();
    }

    let (s, _) = WINDOWS_1252.decode_without_bom_handling(bytes);
    s.into_owned()
}
