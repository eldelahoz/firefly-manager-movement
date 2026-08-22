/// Batch lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BatchStatus {
    Parsed,
    Reconciled,
    Applied,
}

impl BatchStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            BatchStatus::Parsed => "parsed",
            BatchStatus::Reconciled => "reconciled",
            BatchStatus::Applied => "applied",
        }
    }
}

impl std::str::FromStr for BatchStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parsed" => Ok(Self::Parsed),
            "reconciled" => Ok(Self::Reconciled),
            "applied" => Ok(Self::Applied),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn batch_status_roundtrip() {
        for s in ["parsed", "reconciled", "applied"] {
            let parsed = BatchStatus::from_str(s).unwrap();
            assert_eq!(parsed.as_str(), s);
        }
        assert!(BatchStatus::from_str("unknown").is_err());
    }
}
