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
