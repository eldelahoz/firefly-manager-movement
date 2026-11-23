//! Generic description key for counterparty suggestions (RF-08 v0.11).
//! No bank-specific rules: the same normalization applies to any statement.

/// Normalizes a movement description into a matching key: lowercase,
/// accents stripped, punctuation to spaces, digit runs of 4+ dropped
/// (references, dates, card numbers), spaces collapsed and trimmed.
pub fn description_key(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    // `str::to_lowercase` (not per char) gets context-dependent forms like Greek final sigma.
    for c in text.to_lowercase().chars() {
        match fold_latin(c) {
            Some(plain) => folded.push_str(plain),
            // Combining diacritics (decomposed accents) belong to the previous letter.
            None if ('\u{0300}'..='\u{036F}').contains(&c) => {}
            None if c.is_alphanumeric() => folded.push(c),
            None => folded.push(' '),
        }
    }
    folded
        .split(|c: char| !c.is_alphanumeric())
        .flat_map(split_long_digit_runs)
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Splits a word around its runs of 4+ ASCII digits, which are dropped.
fn split_long_digit_runs(word: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut start, mut run_start) = (0, None);
    for (i, c) in word.char_indices().chain([(word.len(), ' ')]) {
        match (c.is_ascii_digit(), run_start) {
            (true, None) => run_start = Some(i),
            (false, Some(r)) => {
                if i - r >= 4 {
                    parts.push(&word[start..r]);
                    start = i;
                }
                run_start = None;
            }
            _ => {}
        }
    }
    parts.push(&word[start..]);
    parts
}

// shortcut: Latin-script diacritics only (Latin-1 + Extended-A); add an NFD
// crate if statements in other diacritic-heavy scripts need folding.
fn fold_latin(c: char) -> Option<&'static str> {
    const FOLDS: &[(&str, &str)] = &[
        ("àáâãäåāăą", "a"),
        ("çćĉċč", "c"),
        ("ďđ", "d"),
        ("èéêëēĕėęě", "e"),
        ("ĝğġģ", "g"),
        ("ĥħ", "h"),
        ("ìíîïĩīĭįı", "i"),
        ("ĵ", "j"),
        ("ķ", "k"),
        ("ĺļľŀł", "l"),
        ("ñńņňŉ", "n"),
        ("òóôõöøōŏő", "o"),
        ("ŕŗř", "r"),
        ("śŝşš", "s"),
        ("ţťŧ", "t"),
        ("ùúûüũūŭůűų", "u"),
        ("ŵ", "w"),
        ("ýÿŷ", "y"),
        ("źżž", "z"),
        ("ß", "ss"),
        ("æ", "ae"),
        ("œ", "oe"),
    ];
    FOLDS
        .iter()
        .find(|(from, _)| from.contains(c))
        .map(|&(_, to)| to)
}

#[cfg(test)]
mod tests {
    use super::description_key;

    #[test]
    fn normalizes_descriptions_from_any_bank_or_language() {
        let cases = [
            // Colombia (PSE)
            (
                "PAGO PSE Acme Motors S.A. 20261117 REF 99812",
                "pago pse acme motors s a ref",
            ),
            (
                "COMPRA EN  ÓPTICA-CALLE 80 *1234",
                "compra en optica calle 80",
            ),
            // Spain
            (
                "Recibo Teléfonos España, S.A.U. 2026/11",
                "recibo telefonos espana s a u 11",
            ),
            // France
            (
                "PRLV SEPA Électricité Générale 0123456789",
                "prlv sepa electricite generale",
            ),
            ("CB CAFÉ CRÈME 12/11", "cb cafe creme 12 11"),
            // Germany
            (
                "Überweisung Müller GmbH Straße",
                "uberweisung muller gmbh strasse",
            ),
            // UK / card numbers glued to words
            (
                "CARD PAYMENT TO CORNER STORES 3412 ON 05-11",
                "card payment to corner stores on 05 11",
            ),
            ("SHOP Mktp REF99812X", "shop mktp ref x"),
            // Portuguese, Polish, Nordic
            ("Transferência João Ação", "transferencia joao acao"),
            ("Płatność Łódź", "platnosc lodz"),
            ("Øresund Æblegård", "oresund aeblegard"),
            // Decomposed accents (NFD, e.g. macOS exports) fold like composed ones
            ("Tele\u{301}fono Cafe\u{301}", "telefono cafe"),
            // Turkish dotted capital I lowercases to i + U+0307
            ("İSTANBUL", "istanbul"),
            // Non-Latin scripts keep their letters; Greek final sigma is consistent
            ("Оплата Связь №123456", "оплата связь"),
            ("ΚΑΦΕΣ", "καφες"),
            ("Καφες", "καφες"),
            // Short numbers stay, long ones go
            ("Tienda 24", "tienda 24"),
            ("  --  ", ""),
            ("20261117", ""),
        ];
        for (input, expected) in cases {
            assert_eq!(description_key(input), expected, "input: {input:?}");
        }
    }

    #[test]
    fn same_merchant_with_different_references_gets_the_same_key() {
        assert_eq!(
            description_key("PAGO PSE Acme Motors S.A. 20261017 REF 77100"),
            description_key("pago pse ACME MOTORS s.a. 20261117 ref 99812"),
        );
    }
}
