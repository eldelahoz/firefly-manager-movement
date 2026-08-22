use ffm_application::ports::CsvDecoder;
use ffm_infrastructure::csv::EncodingCsvDecoder;

#[tokio::test]
async fn decode_utf8_without_bom() {
    let decoder = EncodingCsvDecoder;
    let input = "date,amount,description\n2026-01-05,100.00,Coffee"
        .as_bytes()
        .to_vec();
    let result = decoder.decode(input).await.unwrap();
    assert_eq!(result, "date,amount,description\n2026-01-05,100.00,Coffee");
}

#[tokio::test]
async fn decode_utf8_with_bom_strips_bom() {
    let decoder = EncodingCsvDecoder;
    let mut input = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    input.extend_from_slice(b"date,amount\n2026-01-05,100.00");
    let result = decoder.decode(input).await.unwrap();
    assert_eq!(result, "date,amount\n2026-01-05,100.00");
}

#[tokio::test]
async fn decode_windows1252_french_accents() {
    let decoder = EncodingCsvDecoder;
    // "libellé" in windows-1252: 0xE9 = é
    let input = vec![b'l', b'i', b'b', b'e', b'l', b'l', 0xE9];
    let result = decoder.decode(input).await.unwrap();
    assert_eq!(result, "libellé");
}

#[tokio::test]
async fn decode_utf16le_with_bom() {
    let decoder = EncodingCsvDecoder;
    let mut input = vec![0xFF, 0xFE]; // UTF-16LE BOM
                                      // "hi" in UTF-16LE: h=0x68 0x00, i=0x69 0x00
    input.extend_from_slice(&[0x68, 0x00, 0x69, 0x00]);
    let result = decoder.decode(input).await.unwrap();
    assert_eq!(result, "hi");
}

#[tokio::test]
async fn decode_utf16be_with_bom() {
    let decoder = EncodingCsvDecoder;
    let mut input = vec![0xFE, 0xFF]; // UTF-16BE BOM
                                      // "hi" in UTF-16BE: h=0x00 0x68, i=0x00 0x69
    input.extend_from_slice(&[0x00, 0x68, 0x00, 0x69]);
    let result = decoder.decode(input).await.unwrap();
    assert_eq!(result, "hi");
}

#[tokio::test]
async fn decode_empty_bytes() {
    let decoder = EncodingCsvDecoder;
    let result = decoder.decode(vec![]).await.unwrap();
    assert_eq!(result, "");
}
