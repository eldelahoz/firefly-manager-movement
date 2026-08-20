//! Value objects del dominio.

use std::str::FromStr;

/// Monto monetario en centavos con signo.
/// - Valor positivo = crédito (depósito).
/// - Valor negativo = débito (retraito/withdrawal).
///   (TRD D2: BIGINT signé, sens dérivé du signe — sin enum Direction, sin floats.)
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

    /// true si el monto es negativo (débito / retraito).
    pub fn is_debit(self) -> bool {
        self.0 < 0
    }

    /// true si el monto es positivo (crédito / dépósito).
    pub fn is_credit(self) -> bool {
        self.0 > 0
    }
}

impl FromStr for AmountCents {
    type Err = AmountParseError;

    /// Parsea una cadena de monto a centavos con signo.
    ///
    /// Acepta:
    /// - separador decimal `,` (EU) o `.` (US)
    /// - separador de miles opcional (agrupaciones de 3 dígitos)
    /// - signo `-` ASCII o `−` U+2212
    /// - sin decimales (se asume valor en unidades enteras → ×100)
    ///
    /// Reglas de detección de decimales:
    /// - Sin separadores: entero (×100).
    /// - Un solo separador con exactamente 2 dígitos tras él → decimales.
    /// - Un solo separador con 3 dígitos tras él → miles (entero ×100).
    /// - Ambos separadores: el último es decimal (2 dígitos tras él), el otro miles
    ///   (agrupaciones de exactamente 3 dígitos).
    /// - Cualquier otra combinación (p. ej. `"12,3,4"`) → `InvalidFormat`.
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
                if after.len() != 2 {
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
                (
                    before.chars().filter(|c| c.is_ascii_digit()).collect(),
                    after.to_string(),
                )
            }
            (c, d) => {
                let sep = if c > 0 { ',' } else { '.' };
                let n = if c > 0 { c } else { d };
                let idx = rest.rfind(sep).unwrap();
                let before = &rest[..idx];
                let after = &rest[idx + 1..];
                match after.len() {
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

/// Valida agrupaciones de miles: la primera puede tener 1-3 dígitos, el resto exactamente 3.
fn is_valid_thousands(s: &str, sep: char) -> bool {
    let mut groups = s.split(sep);
    match groups.next() {
        Some(first) if !first.is_empty() && first.len() <= 3 => {}
        _ => return false,
    }
    groups.all(|g| g.len() == 3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn new_and_accessors() {
        let a = AmountCents::new(1234);
        assert_eq!(a.as_i64(), 1234);
        assert!(!a.is_debit() && a.is_credit());
        let b = AmountCents::new(-1234);
        assert!(b.is_debit() && !b.is_credit());
        let z = AmountCents::new(0);
        assert!(!z.is_debit() && !z.is_credit());
    }

    #[test]
    fn parse_eu_comma_decimal() {
        // "12,34" -> 1234 céntimos
        assert_eq!(AmountCents::from_str("12,34").map(|a| a.as_i64()), Ok(1234));
        assert_eq!(
            AmountCents::from_str("-12,34").map(|a| a.as_i64()),
            Ok(-1234)
        );
    }

    #[test]
    fn parse_us_dot_decimal() {
        // "12.34" -> 1234 céntimos
        assert_eq!(AmountCents::from_str("12.34").map(|a| a.as_i64()), Ok(1234));
        assert_eq!(
            AmountCents::from_str("-12.34").map(|a| a.as_i64()),
            Ok(-1234)
        );
    }

    #[test]
    fn parse_unicode_minus() {
        // "−12,34" (U+2212) -> -1234
        assert_eq!(
            AmountCents::from_str("\u{2212}12,34").map(|a| a.as_i64()),
            Ok(-1234)
        );
    }

    #[test]
    fn parse_thousands_separator_optional() {
        // "1.234,56" (EU: miles con punto, decimales con coma) -> 123456
        assert_eq!(
            AmountCents::from_str("1.234,56").map(|a| a.as_i64()),
            Ok(123456)
        );
        // "1,234.56" (US: miles con coma, decimales con punto) -> 123456
        assert_eq!(
            AmountCents::from_str("1,234.56").map(|a| a.as_i64()),
            Ok(123456)
        );
    }

    #[test]
    fn parse_integer_no_decimals() {
        assert_eq!(AmountCents::from_str("100").map(|a| a.as_i64()), Ok(10000));
        assert_eq!(
            AmountCents::from_str("-100").map(|a| a.as_i64()),
            Ok(-10000)
        );
    }

    #[test]
    fn parse_empty_and_garbage() {
        assert_eq!(AmountCents::from_str(""), Err(AmountParseError::Empty));
        assert_eq!(
            AmountCents::from_str("abc"),
            Err(AmountParseError::InvalidFormat)
        );
        assert_eq!(
            AmountCents::from_str("12,3,4"),
            Err(AmountParseError::InvalidFormat)
        );
    }

    #[test]
    fn parse_preserves_sign_and_sens() {
        let neg = AmountCents::from_str("-50,00").unwrap();
        assert!(neg.is_debit());
        let pos = AmountCents::from_str("50,00").unwrap();
        assert!(pos.is_credit());
    }
}
