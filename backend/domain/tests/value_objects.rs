//! Value object unit tests (moved out of src/ — see tests/ folder convention).

use std::str::FromStr;

use ffm_domain::value_objects::{
    AmountCents, AmountParseError, BatchStatus, HeaderSignature, MovementStatus,
};

// ── AmountCents ─────────────────────────────────────────────────────────

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
    // "12,34" -> 1234 cents
    assert_eq!(AmountCents::from_str("12,34").map(|a| a.as_i64()), Ok(1234));
    assert_eq!(
        AmountCents::from_str("-12,34").map(|a| a.as_i64()),
        Ok(-1234)
    );
}

#[test]
fn parse_us_dot_decimal() {
    // "12.34" -> 1234 cents
    assert_eq!(AmountCents::from_str("12.34").map(|a| a.as_i64()), Ok(1234));
    assert_eq!(
        AmountCents::from_str("-12.34").map(|a| a.as_i64()),
        Ok(-1234)
    );
}

#[test]
fn parse_one_decimal_digit_is_padded() {
    // Some exports use a single decimal digit: "23,4" == 23.40
    assert_eq!(AmountCents::from_str("23,4").map(|a| a.as_i64()), Ok(2340));
    assert_eq!(AmountCents::from_str("23.4").map(|a| a.as_i64()), Ok(2340));
    assert_eq!(
        AmountCents::from_str("-23,4").map(|a| a.as_i64()),
        Ok(-2340)
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
    // "1.234,56" (EU: dot for thousands, comma for decimals) -> 123456
    assert_eq!(
        AmountCents::from_str("1.234,56").map(|a| a.as_i64()),
        Ok(123456)
    );
    // "1,234.56" (US: comma for thousands, dot for decimals) -> 123456
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

// ── MovementStatus ──────────────────────────────────────────────────────

#[test]
fn movement_status_roundtrip() {
    for s in ["imported", "matched", "missing", "created", "ignored"] {
        let parsed = MovementStatus::from_str(s).unwrap();
        assert_eq!(parsed.as_str(), s);
    }
    assert!(MovementStatus::from_str("unknown").is_err());
}

// ── BatchStatus ─────────────────────────────────────────────────────────

#[test]
fn batch_status_roundtrip() {
    for s in ["parsed", "reconciled", "applied"] {
        let parsed = BatchStatus::from_str(s).unwrap();
        assert_eq!(parsed.as_str(), s);
    }
    assert!(BatchStatus::from_str("unknown").is_err());
}

// ── HeaderSignature ─────────────────────────────────────────────────────

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
