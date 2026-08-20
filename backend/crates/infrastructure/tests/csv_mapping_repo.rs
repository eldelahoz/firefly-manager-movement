mod common;

use ffm_application::dtos::NewCsvMapping;
use ffm_application::ports::CsvMappingRepository;
use ffm_domain::value_objects::HeaderSignature;
use ffm_infrastructure::persistence::repositories::csv_mapping::MysqlCsvMappingRepository;

fn signature() -> HeaderSignature {
    HeaderSignature::from_headers(&[
        "Date".to_string(),
        "Amount".to_string(),
        "Description".to_string(),
    ])
}

#[tokio::test]
async fn upsert_and_by_signature_idempotent() {
    let pool = common::setup_pool().await;
    common::clean(&pool).await;
    let repo = MysqlCsvMappingRepository::new(pool.clone());

    let sig = signature();
    assert!(repo
        .by_signature(sig.clone())
        .await
        .expect("none")
        .is_none());

    let id1 = repo
        .upsert(NewCsvMapping {
            header_signature: sig.clone(),
            mapping_json: r#"{"amount":"monto"}"#.into(),
        })
        .await
        .expect("upsert 1");

    let got = repo
        .by_signature(sig.clone())
        .await
        .expect("got")
        .expect("some");
    assert_eq!(got.id, id1);
    assert_eq!(got.header_signature, sig);
    assert_eq!(got.mapping_json, r#"{"amount":"monto"}"#);

    // Misma firma con otro mapping: misma fila, mapping actualizado.
    let id2 = repo
        .upsert(NewCsvMapping {
            header_signature: signature(),
            mapping_json: r#"{"amount":"importe","date":"fecha"}"#.into(),
        })
        .await
        .expect("upsert 2");
    assert_eq!(id2, id1);

    let updated = repo
        .by_signature(signature())
        .await
        .expect("updated")
        .expect("some");
    assert_eq!(updated.id, id1);
    assert_eq!(
        updated.mapping_json,
        r#"{"amount":"importe","date":"fecha"}"#
    );

    common::clean(&pool).await;
}
