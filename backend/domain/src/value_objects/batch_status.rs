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
