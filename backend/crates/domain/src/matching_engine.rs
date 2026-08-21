//! Rapprochement engine (domain service, pure function).
//!
//! Pairs imported movements with Firefly III transactions by signed
//! amount in cents (the sign encodes the sense) within a date
//! tolerance, 1-to-1 chronological. Surplus FF3 transactions inside
//! the window of some movement are flagged as duplicates (RF-15).

use chrono::NaiveDate;
use std::collections::HashMap;

use crate::entities::{Ff3Transaction, Movement};
use crate::ids::{Ff3TransactionId, MovementId};

/// Paired movement ↔ FF3 transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchPair {
    pub movement_id: MovementId,
    pub ff3_transaction_id: Ff3TransactionId,
}

/// Result of a rapprochement run.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MatchResult {
    pub matches: Vec<MatchPair>,
    pub missing: Vec<MovementId>,
    pub extra: Vec<Ff3TransactionId>,
    pub duplicates: Vec<Ff3TransactionId>,
}

/// Runs the matching engine.
///
/// - `movements`: movements of the batch. The caller must exclude `ignored` ones.
/// - `ff3_transactions`: FF3 transactions of the target account within the window.
/// - `tolerance_days`: ±N days tolerance on the date.
///
/// Algorithm (TRD §6, PRD §4):
/// 1. Index by signed `amount_cents` (the sign encodes the sense).
/// 2. Per key, pair 1-to-1 chronologically within the tolerance.
/// 3. Batch surplus → `missing`; FF3 surplus → `extra` or `duplicate`.
/// 4. An unpaired FF3 is `duplicate` if it is within the tolerance of
///    some movement in its group; otherwise, `extra`.
///
/// Outputs sorted by id for determinism.
pub fn run_matching(
    movements: &[Movement],
    ff3_transactions: &[Ff3Transaction],
    tolerance_days: i32,
) -> MatchResult {
    let tol = tolerance_days as i64;

    // Index by signed amount_cents (the sign encodes the sense).
    let mut by_amount: HashMap<i64, (Vec<usize>, Vec<usize>)> = HashMap::new();
    for (i, m) in movements.iter().enumerate() {
        by_amount
            .entry(m.amount_cents.as_i64())
            .or_default()
            .0
            .push(i);
    }
    for (i, t) in ff3_transactions.iter().enumerate() {
        by_amount
            .entry(t.amount_cents.as_i64())
            .or_default()
            .1
            .push(i);
    }

    let mut matches = Vec::new();
    let mut missing = Vec::new();
    let mut extra = Vec::new();
    let mut duplicates = Vec::new();

    for (_, (mut m_idx, mut t_idx)) in by_amount {
        // Sort by ascending date (chronological).
        m_idx.sort_by_key(|&i| movements[i].date);
        t_idx.sort_by_key(|&i| ff3_transactions[i].date);

        let mut t_used = vec![false; t_idx.len()];

        // Greedy 1-to-1: each movement with the first unused FF3 within the window.
        for &mi in &m_idx {
            let m_date = movements[mi].date;
            let paired = t_idx
                .iter()
                .enumerate()
                .find(|(j, &ti)| !t_used[*j] && within_tol(ff3_transactions[ti].date, m_date, tol));
            match paired {
                Some((j, &ti)) => {
                    t_used[j] = true;
                    matches.push(MatchPair {
                        movement_id: movements[mi].id,
                        ff3_transaction_id: ff3_transactions[ti].id,
                    });
                }
                None => missing.push(movements[mi].id),
            }
        }

        // Classify unmatched FF3: duplicate if some movement is near, otherwise extra.
        for (j, &ti) in t_idx.iter().enumerate() {
            if t_used[j] {
                continue;
            }
            let t_date = ff3_transactions[ti].date;
            let near_some_movement = m_idx
                .iter()
                .any(|&mi| within_tol(t_date, movements[mi].date, tol));
            if near_some_movement {
                duplicates.push(ff3_transactions[ti].id);
            } else {
                extra.push(ff3_transactions[ti].id);
            }
        }
    }

    // Determinism: sort outputs by id.
    matches.sort_by_key(|p| (p.movement_id, p.ff3_transaction_id));
    missing.sort();
    extra.sort();
    duplicates.sort();

    MatchResult {
        matches,
        missing,
        extra,
        duplicates,
    }
}

fn within_tol(a: NaiveDate, b: NaiveDate, tol: i64) -> bool {
    (a - b).num_days().abs() <= tol
}

#[cfg(test)]
mod tests {
    use super::{run_matching, MatchPair};
    use crate::entities::{Ff3Transaction, Movement};
    use crate::ids::{BatchId, Ff3TransactionId, MovementId, ReconciliationId};
    use crate::value_objects::{AmountCents, MovementStatus};
    use chrono::{NaiveDate, NaiveDateTime};

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
}
