use std::str::FromStr;

/// Explicit date format for CSV parsing (RF-02 v0.8).
///
/// Wire representation uses lowercase strings such as `dd/mm/yyyy`.
/// When a mapping carries a format, the parser uses ONLY it; without one
/// it falls back to the legacy ordered auto-try of these six formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DateFormat {
    DdMmYyyySlash,
    MmDdYyyySlash,
    YyyyMmDdDash,
    DdMmYyyyDot,
    DdMmYyyyDash,
    YyyyMmDdSlash,
}

impl DateFormat {
    /// All known formats, in the legacy auto-try order.
    pub const ALL: [DateFormat; 6] = [
        DateFormat::YyyyMmDdDash,
        DateFormat::DdMmYyyySlash,
        DateFormat::DdMmYyyyDot,
        DateFormat::DdMmYyyyDash,
        DateFormat::MmDdYyyySlash,
        DateFormat::YyyyMmDdSlash,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            DateFormat::DdMmYyyySlash => "dd/mm/yyyy",
            DateFormat::MmDdYyyySlash => "mm/dd/yyyy",
            DateFormat::YyyyMmDdDash => "yyyy-mm-dd",
            DateFormat::DdMmYyyyDot => "dd.mm.yyyy",
            DateFormat::DdMmYyyyDash => "dd-mm-yyyy",
            DateFormat::YyyyMmDdSlash => "yyyy/mm/dd",
        }
    }

    /// The chrono pattern this format maps to.
    pub fn chrono_format(&self) -> &'static str {
        match self {
            DateFormat::DdMmYyyySlash => "%d/%m/%Y",
            DateFormat::MmDdYyyySlash => "%m/%d/%Y",
            DateFormat::YyyyMmDdDash => "%Y-%m-%d",
            DateFormat::DdMmYyyyDot => "%d.%m.%Y",
            DateFormat::DdMmYyyyDash => "%d-%m-%Y",
            DateFormat::YyyyMmDdSlash => "%Y/%m/%d",
        }
    }

    /// True when day-first and month-first readings of the same value are
    /// both valid dates, i.e. the format alone cannot disambiguate samples.
    pub fn is_ambiguous(&self) -> bool {
        matches!(self, DateFormat::DdMmYyyySlash | DateFormat::MmDdYyyySlash)
    }
}

impl FromStr for DateFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_lowercase().as_str() {
            "dd/mm/yyyy" => DateFormat::DdMmYyyySlash,
            "mm/dd/yyyy" => DateFormat::MmDdYyyySlash,
            "yyyy-mm-dd" => DateFormat::YyyyMmDdDash,
            "dd.mm.yyyy" => DateFormat::DdMmYyyyDot,
            "dd-mm-yyyy" => DateFormat::DdMmYyyyDash,
            "yyyy/mm/dd" => DateFormat::YyyyMmDdSlash,
            _ => return Err(()),
        })
    }
}
