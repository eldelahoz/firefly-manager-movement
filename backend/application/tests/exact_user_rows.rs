use ffm_application::csv::{parse_csv, ColumnMapping};

#[test]
fn exact_user_rows() {
    let csv = "Fecha;Descripción;Referencia;Valor\n03/08/2026;COMPRA EN X COMERCIO;;-4200\n03/08/2026;COMPRA EN X CAFE;;-6300\n03/08/2026;PAGO X PLATAFORMA;536486978;-41900\n02/08/2026;ABONO INTERESES;;6,85";

    let mapping = ColumnMapping {
        date_column: "Fecha".into(),
        amount_column: "Valor".into(),
        description_column: "Descripción".into(),
        origin_account_column: None,
        date_format: None,
    };

    let r = parse_csv(csv, &mapping).unwrap();

    for m in &r.valid {
        eprintln!(
            "line {} date {} amount_cents {} desc {}",
            m.line_no,
            m.date,
            m.amount_cents.as_i64(),
            m.description
        );
    }
    assert_eq!(r.valid.len(), 4);
    assert_eq!(r.valid[0].amount_cents.as_i64(), -420000); // -4200.00
    assert_eq!(r.valid[1].amount_cents.as_i64(), -630000); // -6300.00
    assert_eq!(r.valid[2].amount_cents.as_i64(), -4190000); // -41900.00
    assert_eq!(r.valid[3].amount_cents.as_i64(), 685); // 6.85
}
