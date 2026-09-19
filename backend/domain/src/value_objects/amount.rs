use std::str::FromStr;

/// Monetary amount in signed cents.
/// - Positive value = credit (deposit).
/// - Negative value = debit (withdrawal).
///   (TRD D2: signed BIGINT, sense derived from the sign - no Direction enum, no floats.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AmountCents(i64);

#[derive(Debug, PartialEq, Eq)]
pub enum AmountParseError {
    Empty,
    InvalidFormat,
}

impl AmountCents {
    pub fn new(cents: i64) -> Self {
        Self(cents)
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }

    /// true if the amount is negative (debit / withdrawal).
    pub fn is_debit(self) -> bool {
        self.0 < 0
    }

    /// true if the amount is positive (credit / deposit).
    pub fn is_credit(self) -> bool {
        self.0 > 0
    }

    /// Parses an FF3 API amount string (machine format) into signed cents.
    ///
    /// FF3 returns amounts with arbitrary decimal precision (e.g.
    /// `"1000.000000000000"`), unlike CSV files - do not use [`FromStr`] here,
    /// its separator heuristics reject >3 decimal digits.
    ///
    /// Accepts: optional `-` sign, digits, optional `.` followed by any number
    /// of digits. Rounded to cents, half away from zero (3rd decimal ≥ 5 → 1 cent).
    pub fn from_api_str(s: &str) -> Result<Self, AmountParseError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(AmountParseError::Empty);
        }
        let (neg, rest) = if let Some(r) = s.strip_prefix('-') {
            (true, r)
        } else {
            (false, s)
        };
        let (int_part, frac_part) = match rest.split_once('.') {
            Some((i, f)) => (i, f),
            None => (rest, ""),
        };
        if int_part.is_empty()
            || !int_part.chars().all(|c| c.is_ascii_digit())
            || !frac_part.chars().all(|c| c.is_ascii_digit())
        {
            return Err(AmountParseError::InvalidFormat);
        }

        let int_val: i64 = int_part
            .parse()
            .map_err(|_| AmountParseError::InvalidFormat)?;
        let mut cents = int_val
            .checked_mul(100)
            .ok_or(AmountParseError::InvalidFormat)?;
        if !frac_part.is_empty() {
            let mut frac = frac_part.to_string();
            frac.truncate(2);
            while frac.len() < 2 {
                frac.push('0');
            }
            let frac_val: i64 = frac.parse().map_err(|_| AmountParseError::InvalidFormat)?;
            cents = cents
                .checked_add(frac_val)
                .ok_or(AmountParseError::InvalidFormat)?;
        }
        if matches!(frac_part.as_bytes().get(2), Some(d) if *d >= b'5') {
            cents = cents
                .checked_add(1)
                .ok_or(AmountParseError::InvalidFormat)?;
        }
        Ok(AmountCents(if neg { -cents } else { cents }))
    }
}

impl FromStr for AmountCents {
    type Err = AmountParseError;

    /// Parses an amount string into signed cents.
    ///
    /// Accepts:
    /// - decimal separator `,` (EU) or `.` (US)
    /// - optional thousands separator (groups of 3 digits)
    /// - `-` ASCII sign or `−` U+2212
    /// - no decimals (value assumed in whole units → ×100)
    ///
    /// Decimal detection rules:
    /// - No separators: integer (×100).
    /// - A single separator with exactly 2 digits after it → decimals.
    /// - A single separator with 3 digits after it → thousands (integer ×100).
    /// - Both separators: the last one is decimal (2 digits after it), the other is thousands
    ///   (groups of exactly 3 digits).
    /// - Any other combination (e.g. `"12,3,4"`) → `InvalidFormat`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(AmountParseError::Empty);
        }

        let (neg, rest) = if let Some(r) = s.strip_prefix('-') {
            (true, r)
        } else if let Some(r) = s.strip_prefix('\u{2212}') {
            (true, r)
        } else {
            (false, s)
        };

        if rest.is_empty()
            || rest
                .chars()
                .any(|c| !c.is_ascii_digit() && c != ',' && c != '.')
        {
            return Err(AmountParseError::InvalidFormat);
        }

        let comma_count = rest.matches(',').count();
        let dot_count = rest.matches('.').count();

        let (int_part, frac_part): (String, String) = match (comma_count, dot_count) {
            (0, 0) => (rest.to_string(), "00".to_string()),
            (c, d) if c > 0 && d > 0 => {
                let idx = rest.rfind([',', '.']).unwrap();
                let before = &rest[..idx];
                let after = &rest[idx + 1..];
                if after.len() != 1 && after.len() != 2 {
                    return Err(AmountParseError::InvalidFormat);
                }
                let other_sep = if rest.as_bytes()[idx] == b',' {
                    '.'
                } else {
                    ','
                };
                if !is_valid_thousands(before, other_sep) {
                    return Err(AmountParseError::InvalidFormat);
                }
                let frac = if after.len() == 1 {
                    format!("{}0", after)
                } else {
                    after.to_string()
                };
                (
                    before.chars().filter(|c| c.is_ascii_digit()).collect(),
                    frac,
                )
            }
            (c, d) => {
                let sep = if c > 0 { ',' } else { '.' };
                let n = if c > 0 { c } else { d };
                let idx = rest.rfind(sep).unwrap();
                let before = &rest[..idx];
                let after = &rest[idx + 1..];
                match after.len() {
                    1 => {
                        if before.is_empty() {
                            return Err(AmountParseError::InvalidFormat);
                        }
                        if n > 1 && !is_valid_thousands(before, sep) {
                            return Err(AmountParseError::InvalidFormat);
                        }
                        (
                            before.chars().filter(|c| c.is_ascii_digit()).collect(),
                            format!("{}0", after),
                        )
                    }
                    2 => {
                        if before.is_empty() {
                            return Err(AmountParseError::InvalidFormat);
                        }
                        if n > 1 && !is_valid_thousands(before, sep) {
                            return Err(AmountParseError::InvalidFormat);
                        }
                        (
                            before.chars().filter(|c| c.is_ascii_digit()).collect(),
                            after.to_string(),
                        )
                    }
                    3 => {
                        if before.is_empty() || !is_valid_thousands(before, sep) {
                            return Err(AmountParseError::InvalidFormat);
                        }
                        (
                            rest.chars().filter(|c| c.is_ascii_digit()).collect(),
                            "00".to_string(),
                        )
                    }
                    _ => return Err(AmountParseError::InvalidFormat),
                }
            }
        };

        let int_val: i64 = int_part
            .parse()
            .map_err(|_| AmountParseError::InvalidFormat)?;
        let frac_val: i64 = frac_part
            .parse()
            .map_err(|_| AmountParseError::InvalidFormat)?;
        let cents = int_val
            .checked_mul(100)
            .and_then(|v| v.checked_add(frac_val))
            .ok_or(AmountParseError::InvalidFormat)?;
        Ok(AmountCents(if neg { -cents } else { cents }))
    }
}

// Validates thousands groupings: the first may have 1-3 digits, the rest exactly 3.
fn is_valid_thousands(s: &str, sep: char) -> bool {
    let mut groups = s.split(sep);
    match groups.next() {
        Some(first) if !first.is_empty() && first.len() <= 3 => {}
        _ => return false,
    }
    groups.all(|g| g.len() == 3)
}
