/// Signature of a CSV's headers, used to memorize the mapping (RF-02).
/// Computed by normalizing (trim + lowercase) each header and joining with `|`.
/// Deterministic and stable across versions (no cryptographic hash).
///
/// **Header order is significant** (`["a","b"]` ≠ `["b","a"]`).
/// Caveat: if a header contains `|` there can be a theoretical collision (rare in bank CSVs).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HeaderSignature(String);

impl HeaderSignature {
    /// Computes the signature from the CSV header list.
    pub fn from_headers(headers: &[String]) -> Self {
        let normalized: Vec<String> = headers.iter().map(|h| h.trim().to_lowercase()).collect();
        Self(normalized.join("|"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::str::FromStr for HeaderSignature {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_signature_normalizes() {
        let sig = HeaderSignature::from_headers(&[
            "Date".to_string(),
            "  Amount ".to_string(),
            "DESCRIPTION".to_string(),
        ]);
        assert_eq!(sig.as_str(), "date|amount|description");
    }

    #[test]
    fn header_signature_distinct_headers_distinct_sig() {
        let a = HeaderSignature::from_headers(&["a".to_string(), "b".to_string()]);
        let b = HeaderSignature::from_headers(&["b".to_string(), "a".to_string()]);
        assert_ne!(a, b, "el orden de headers importa");
    }
}
