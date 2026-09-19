use serde::{Deserialize, Serialize};

use ffm_domain::value_objects::DateFormat;

/// Maps CSV column names to domain fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub date_column: String,
    pub amount_column: String,
    pub description_column: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_account_column: Option<String>,
    /// Explicit date format for this mapping, memorized by header signature.
    ///
    /// `None` = no explicit format: the settings default applies, then the
    /// legacy ordered auto-try. (RF-02 v0.8)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "date_format_serde"
    )]
    pub date_format: Option<DateFormat>,
}

/// Serde helpers: the VO lives in the domain crate (no serde dep there),
/// so (de)serialization to its wire string happens here.
mod date_format_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;

    use ffm_domain::value_objects::DateFormat;

    pub(super) fn serialize<S: Serializer>(
        f: &Option<DateFormat>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        match f {
            Some(f) => f.as_str().serialize(s),
            None => s.serialize_none(),
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<DateFormat>, D::Error> {
        match Option::<String>::deserialize(d)? {
            None => Ok(None),
            Some(raw) => DateFormat::from_str(&raw)
                .map(Some)
                .map_err(|_| serde::de::Error::custom(format!("unknown date format '{raw}'"))),
        }
    }
}

/// Tries to auto-detect the column mapping from header names.
/// Returns `None` if any of the three required columns cannot be guessed.
/// The origin-account column is optional — auto-detected if found, `None` otherwise.
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

    let origin_account_column = headers
        .iter()
        .find(|h| {
            let l = h.trim().to_lowercase();
            l.contains("account")
                || l.contains("compte")
                || l.contains("cuenta")
                || l.contains("origin")
                || l.contains("origen")
                || l.contains("source")
                || l.contains("emisor")
        })
        .cloned();

    Some(ColumnMapping {
        date_column,
        amount_column,
        description_column,
        origin_account_column,
        date_format: None,
    })
}
