/// Status of an imported movement (PRD §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MovementStatus {
    Imported,
    Matched,
    Missing,
    Created,
    Ignored,
}

impl MovementStatus {
    /// Text representation for persistence (VARCHAR in DB).
    pub fn as_str(self) -> &'static str {
        match self {
            MovementStatus::Imported => "imported",
            MovementStatus::Matched => "matched",
            MovementStatus::Missing => "missing",
            MovementStatus::Created => "created",
            MovementStatus::Ignored => "ignored",
        }
    }
}

impl std::str::FromStr for MovementStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "imported" => Ok(Self::Imported),
            "matched" => Ok(Self::Matched),
            "missing" => Ok(Self::Missing),
            "created" => Ok(Self::Created),
            "ignored" => Ok(Self::Ignored),
            _ => Err(()),
        }
    }
}
