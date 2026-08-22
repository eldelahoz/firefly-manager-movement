use serde::{Deserialize, Serialize};

/// Maps CSV column names to domain fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub date_column: String,
    pub amount_column: String,
    pub description_column: String,
}

/// Tries to auto-detect the column mapping from header names.
/// Returns `None` if any of the three columns cannot be guessed.
pub fn auto_detect_mapping(headers: &[String]) -> Option<ColumnMapping> {
    let date_column = headers
        .iter()
        .find(|h| {
            let l = h.trim().to_lowercase();
            l.contains("date") || l.contains("fecha")
        })
        .cloned()?;

    let amount_column = headers
        .iter()
        .find(|h| {
            let l = h.trim().to_lowercase();
            l.contains("amount")
                || l.contains("montant")
                || l.contains("importe")
                || l.contains("valor")
                || l.contains("montante")
                || l.contains("value")
        })
        .cloned()?;

    let description_column = headers
        .iter()
        .find(|h| {
            let l = h.trim().to_lowercase();
            l.contains("description")
                || l.contains("descrip")
                || l.contains("libell")
                || l.contains("label")
                || l.contains("memo")
                || l.contains("detail")
                || l.contains("narrative")
        })
        .cloned()?;

    Some(ColumnMapping {
        date_column,
        amount_column,
        description_column,
    })
}
