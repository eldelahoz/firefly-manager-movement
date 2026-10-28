//! Matching engine unit tests (moved out of src/ - see tests/ folder convention).

use chrono::{NaiveDate, NaiveDateTime};

use ffm_domain::entities::{Ff3Transaction, Movement};
use ffm_domain::ids::{BatchId, Ff3TransactionId, MovementId, ReconciliationId};
use ffm_domain::matching_engine::{run_matching, MatchPair};
use ffm_domain::value_objects::{AmountCents, MovementStatus};

fn dt() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2000, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

fn mv(id: i64, date: &str, cents: i64) -> Movement {
    Movement {
        id: MovementId::new(id),
        batch_id: BatchId::new(1),
        line_no: 0,
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        amount_cents: AmountCents::new(cents),
        description: String::new(),
        raw_json: String::new(),
        status: MovementStatus::Imported,
        created_at: dt(),
    }
}

fn ff3(id: i64, date: &str, cents: i64) -> Ff3Transaction {
    Ff3Transaction {
        ff3_journal_id: 0,
        id: Ff3TransactionId::new(id),
        reconciliation_id: ReconciliationId::new(1),
        ff3_id: id * 1000,
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        amount_cents: AmountCents::new(cents),
        description: String::new(),
        is_duplicate: false,
        created_at: dt(),
    }
}

#[test]
fn exact_match_one_to_one() {
    let m = [mv(1, "2026-01-05", 10000)];
    let t = [ff3(10, "2026-01-05", 10000)];
    let r = run_matching(&m, &t, 0);
    assert_eq!(r.matches.len(), 1);
    assert_eq!(
        r.matches[0],
        MatchPair {
            movement_id: MovementId::new(1),
            ff3_transaction_id: Ff3TransactionId::new(10)
        }
    );
    assert!(r.missing.is_empty());
    assert!(r.extra.is_empty());
    assert!(r.duplicates.is_empty());
}

#[test]
fn tolerance_plus_one_two_three() {
    let m = [mv(1, "2026-01-05", 10000)];
    // within ±3
    assert_eq!(
        run_matching(&m, &[ff3(10, "2026-01-08", 10000)], 3)
            .matches
            .len(),
        1
    );
    // exactly at the ±2 boundary
    assert_eq!(
        run_matching(&m, &[ff3(10, "2026-01-07", 10000)], 2)
            .matches
            .len(),
        1
    );
    // outside ±1
    let r = run_matching(&m, &[ff3(10, "2026-01-08", 10000)], 1);
    assert!(r.matches.is_empty());
    assert_eq!(r.missing, vec![MovementId::new(1)]);
    assert_eq!(r.extra, vec![Ff3TransactionId::new(10)]);
}

#[test]
fn duplicate_ff3_within_window() {
    // 1 movement, 2 FF3 within the window → 1 match + 1 duplicate
    let m = [mv(1, "2026-01-05", 10000)];
    let t = [ff3(10, "2026-01-05", 10000), ff3(11, "2026-01-06", 10000)];
    let r = run_matching(&m, &t, 2);
    assert_eq!(r.matches.len(), 1);
    assert!(r.missing.is_empty());
    assert!(r.extra.is_empty());
    assert_eq!(r.duplicates, vec![Ff3TransactionId::new(11)]);
}

#[test]
fn batch_surplus_is_missing() {
    let m = [mv(1, "2026-01-05", 10000), mv(2, "2026-01-06", 10000)];
    let t = [ff3(10, "2026-01-05", 10000)];
    let r = run_matching(&m, &t, 2);
    assert_eq!(r.matches.len(), 1);
    assert_eq!(r.missing, vec![MovementId::new(2)]);
    assert!(r.extra.is_empty());
    assert!(r.duplicates.is_empty());
}

#[test]
fn ff3_surplus_far_is_extra() {
    // FF3 with no nearby movement in its group → extra
    let m = [mv(1, "2026-01-05", 10000)];
    let t = [ff3(10, "2026-01-05", 10000), ff3(11, "2026-04-20", 10000)];
    let r = run_matching(&m, &t, 2);
    assert_eq!(r.matches.len(), 1);
    assert!(r.missing.is_empty());
    assert_eq!(r.extra, vec![Ff3TransactionId::new(11)]);
    assert!(r.duplicates.is_empty());
}

#[test]
fn repeated_legitimate_amounts_pair_independently() {
    // 2 movements + 2 FF3 same amount, different dates → 2 matches
    let m = [mv(1, "2026-01-05", 5000), mv(2, "2026-02-05", 5000)];
    let t = [ff3(10, "2026-01-05", 5000), ff3(11, "2026-02-05", 5000)];
    let r = run_matching(&m, &t, 0);
    assert_eq!(r.matches.len(), 2);
    assert!(r.missing.is_empty());
    assert!(r.extra.is_empty());
    assert!(r.duplicates.is_empty());
}

#[test]
fn empty_movements_all_ff3_extra() {
    // with no movements, no FF3 can be a duplicate (no nearby movement)
    let t = [ff3(10, "2026-01-05", 10000), ff3(11, "2026-01-06", 10000)];
    let r = run_matching(&[], &t, 2);
    assert!(r.matches.is_empty());
    assert!(r.missing.is_empty());
    assert_eq!(
        r.extra,
        vec![Ff3TransactionId::new(10), Ff3TransactionId::new(11)]
    );
    assert!(r.duplicates.is_empty());
}

#[test]
fn empty_ff3_all_movements_missing() {
    let m = [mv(1, "2026-01-05", 10000), mv(2, "2026-01-06", 10000)];
    let r = run_matching(&m, &[], 2);
    assert!(r.matches.is_empty());
    assert_eq!(r.missing, vec![MovementId::new(1), MovementId::new(2)]);
    assert!(r.extra.is_empty());
    assert!(r.duplicates.is_empty());
}

#[test]
fn both_empty_is_empty_result() {
    let r = run_matching(&[], &[], 2);
    assert!(r.matches.is_empty());
    assert!(r.missing.is_empty());
    assert!(r.extra.is_empty());
    assert!(r.duplicates.is_empty());
}

#[test]
fn sign_matters_opposite_signs_do_not_match() {
    // movement +100 (credit) vs FF3 -100 (debit) → no match
    let m = [mv(1, "2026-01-05", 10000)];
    let t = [ff3(10, "2026-01-05", -10000)];
    let r = run_matching(&m, &t, 2);
    assert!(r.matches.is_empty());
    assert_eq!(r.missing, vec![MovementId::new(1)]);
    assert_eq!(r.extra, vec![Ff3TransactionId::new(10)]);
}

#[test]
fn multiple_amount_groups_independent() {
    let m = [mv(1, "2026-01-05", 10000), mv(2, "2026-01-05", -5000)];
    let t = [ff3(10, "2026-01-05", 10000), ff3(11, "2026-01-05", -5000)];
    let r = run_matching(&m, &t, 0);
    assert_eq!(r.matches.len(), 2);
    assert!(r.missing.is_empty() && r.extra.is_empty() && r.duplicates.is_empty());
}

#[test]
fn chronological_pairing_does_not_cross() {
    // M1@05, M2@10; T1@06, T2@09; tol=2 → M1-T1, M2-T2 (no crossing)
    let m = [mv(1, "2026-01-05", 10000), mv(2, "2026-01-10", 10000)];
    let t = [ff3(10, "2026-01-06", 10000), ff3(11, "2026-01-09", 10000)];
    let r = run_matching(&m, &t, 2);
    assert_eq!(r.matches.len(), 2);
    assert!(r
        .matches
        .iter()
        .any(|p| p.movement_id.as_i64() == 1 && p.ff3_transaction_id.as_i64() == 10));
    assert!(r
        .matches
        .iter()
        .any(|p| p.movement_id.as_i64() == 2 && p.ff3_transaction_id.as_i64() == 11));
}

#[test]
fn duplicate_of_already_matched_movement() {
    // 2 movements + 3 FF3 (all same amount, all within M1/M2's window)
    let m = [mv(1, "2026-01-05", 10000), mv(2, "2026-01-06", 10000)];
    let t = [
        ff3(10, "2026-01-05", 10000),
        ff3(11, "2026-01-06", 10000),
        ff3(12, "2026-01-07", 10000),
    ];
    let r = run_matching(&m, &t, 2);
    assert_eq!(r.matches.len(), 2);
    assert!(r.missing.is_empty());
    // T12@07 is within M1's ([03,07]) and M2's ([04,08]) window → duplicate
    assert_eq!(r.duplicates, vec![Ff3TransactionId::new(12)]);
    assert!(r.extra.is_empty());
}

#[test]
fn benchmark_1000_movements_under_5s_nfr04() {
    // NFR-04: 1000 movements < 5s excluding latency.
    let n = 1000;
    let base = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    let movements: Vec<Movement> = (0..n)
        .map(|i| Movement {
            id: MovementId::new(i + 1),
            batch_id: BatchId::new(1),
            line_no: i as i32,
            date: base + chrono::Days::new((i % 90) as u64),
            amount_cents: AmountCents::new((i % 500) * 37 - 9000),
            description: String::new(),
            raw_json: String::new(),
            status: MovementStatus::Imported,
            created_at: dt(),
        })
        .collect();

    let ff3_txs: Vec<Ff3Transaction> = (0..n)
        .map(|i| Ff3Transaction {
            ff3_journal_id: 0,
            id: Ff3TransactionId::new(i + 1),
            reconciliation_id: ReconciliationId::new(1),
            ff3_id: (i + 1) * 1000,
            date: base + chrono::Days::new(((i + 3) % 90) as u64),
            amount_cents: AmountCents::new((i % 500) * 37 - 9000),
            description: String::new(),
            is_duplicate: false,
            created_at: dt(),
        })
        .collect();

    let start = std::time::Instant::now();
    let r = run_matching(&movements, &ff3_txs, 3);
    let elapsed = start.elapsed();

    assert_eq!(r.matches.len() + r.missing.len(), n as usize);
    assert!(
        elapsed.as_secs_f64() < 5.0,
        "matching engine took {elapsed:?} for {n} movements (NFR-04 limit: 5s)"
    );
}
