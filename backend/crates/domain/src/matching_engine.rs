//! Motor de rapprochement (servicio de dominio, función pura).
//!
//! Aparea movimientos importados con transacciones de Firefly III por
//! monto en centavos con signo (el signo codifica el sens) dentro de una
//! tolerancia de fecha, 1-a-1 cronológico. Los excedentes FF3 dentro de la
//! ventana de algún movimiento se marcan como duplicados (RF-15).

use chrono::NaiveDate;
use std::collections::HashMap;

use crate::entities::{Ff3Transaction, Movement};
use crate::ids::{Ff3TransactionId, MovementId};

/// Par emparejado movimiento ↔ transacción FF3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchPair {
    pub movement_id: MovementId,
    pub ff3_transaction_id: Ff3TransactionId,
}

/// Resultado de un run de rapprochement.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MatchResult {
    pub matches: Vec<MatchPair>,
    pub missing: Vec<MovementId>,
    pub extra: Vec<Ff3TransactionId>,
    pub duplicates: Vec<Ff3TransactionId>,
}

/// Ejecuta el motor de matching.
///
/// - `movements`: movimientos del batch. El llamador debe excluir los `ignored`.
/// - `ff3_transactions`: transacciones FF3 del cuenta target en la ventana.
/// - `tolerance_days`: tolerancia ±N días sobre la fecha.
///
/// Algoritmo (TRD §6, PRD §4):
/// 1. Indexar por `amount_cents` con signo (el signo codifica el sens).
/// 2. Por clave, emparejar 1-a-1 cronológico dentro de la tolerancia.
/// 3. Excedentes batch → `missing`; excedentes FF3 → `extra` o `duplicate`.
/// 4. Un FF3 sin pareja es `duplicate` si está dentro de la tolerancia de
///    algún movimiento de su grupo; si no, `extra`.
///
/// Salidas ordenadas por id para determinismo.
pub fn run_matching(
    movements: &[Movement],
    ff3_transactions: &[Ff3Transaction],
    tolerance_days: i32,
) -> MatchResult {
    let tol = tolerance_days as i64;

    // Indexar por amount_cents con signo (el signo codifica el sens).
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
        // Ordenar por fecha ascendente (cronológico).
        m_idx.sort_by_key(|&i| movements[i].date);
        t_idx.sort_by_key(|&i| ff3_transactions[i].date);

        let mut t_used = vec![false; t_idx.len()];

        // Greedy 1-a-1: cada movimiento con el primer FF3 no usado dentro de la ventana.
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

        // Clasificar FF3 no emparejados: duplicate si hay algún movimiento cerca, si no extra.
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

    // Determinismo: ordenar salidas por id.
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
        // dentro de ±3
        assert_eq!(
            run_matching(&m, &[ff3(10, "2026-01-08", 10000)], 3)
                .matches
                .len(),
            1
        );
        // exactamente en el límite ±2
        assert_eq!(
            run_matching(&m, &[ff3(10, "2026-01-07", 10000)], 2)
                .matches
                .len(),
            1
        );
        // fuera de ±1
        let r = run_matching(&m, &[ff3(10, "2026-01-08", 10000)], 1);
        assert!(r.matches.is_empty());
        assert_eq!(r.missing, vec![MovementId::new(1)]);
        assert_eq!(r.extra, vec![Ff3TransactionId::new(10)]);
    }

    #[test]
    fn duplicate_ff3_within_window() {
        // 1 movimiento, 2 FF3 dentro de la ventana → 1 match + 1 duplicate
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
        // FF3 sin ningún movimiento cerca en su grupo → extra
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
        // 2 movimientos + 2 FF3 mismo monto, fechas distintas → 2 matches
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
        // sin movimientos, ningún FF3 puede ser duplicate (no hay movimiento cerca)
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
        // movimiento +100 (crédito) vs FF3 -100 (débito) → no matchean
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
        // M1@05, M2@10; T1@06, T2@09; tol=2 → M1-T1, M2-T2 (no cruzan)
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
        // 2 movimientos + 3 FF3 (todos mismo monto, todos dentro de ventana de M1/M2)
        let m = [mv(1, "2026-01-05", 10000), mv(2, "2026-01-06", 10000)];
        let t = [
            ff3(10, "2026-01-05", 10000),
            ff3(11, "2026-01-06", 10000),
            ff3(12, "2026-01-07", 10000),
        ];
        let r = run_matching(&m, &t, 2);
        assert_eq!(r.matches.len(), 2);
        assert!(r.missing.is_empty());
        // T12@07 está dentro de ventana de M1([03,07]) y M2([04,08]) → duplicate
        assert_eq!(r.duplicates, vec![Ff3TransactionId::new(12)]);
        assert!(r.extra.is_empty());
    }
}
