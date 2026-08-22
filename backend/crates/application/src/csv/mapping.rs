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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_detect_english() {
        let h = vec!["Date".into(), "Amount".into(), "Description".into()];
        let m = auto_detect_mapping(&h).unwrap();
        assert_eq!(m.date_column, "Date");
        assert_eq!(m.amount_column, "Amount");
        assert_eq!(m.description_column, "Description");
    }

    #[test]
    fn auto_detect_french() {
        let h = vec!["Date".into(), "Libellé".into(), "Montant".into()];
        let m = auto_detect_mapping(&h).unwrap();
        assert_eq!(m.date_column, "Date");
        assert_eq!(m.amount_column, "Montant");
        assert_eq!(m.description_column, "Libellé");
    }

    #[test]
    fn auto_detect_none_for_unknown() {
        let h = vec!["col1".into(), "col2".into(), "col3".into()];
        assert!(auto_detect_mapping(&h).is_none());
    }
}
