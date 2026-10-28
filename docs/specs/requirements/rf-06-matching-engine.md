# RF-06 - Matching Engine

| Field | Value |
| --- | --- |
| Version | v0.10 (integrates PRD v0.10 amendment) |
| Priority | P0 |
| Use cases | CU-3 |
| Depends on | RF-02 (parsed movements), RF-05 (FF3 transactions) |
| Related | RF-07 (result groups), RF-10 (ignored exclusion), RF-12 (currency check), RF-13 (re-run), RF-15 (duplicate flagging), NFR-04 (performance), NFR-06 (amounts as cents) |

## Description

The reconciliation engine compares the batch movements with the FF3 transactions of the target account using four criteria:

1. **Amount**: exact equality - amounts are compared exactly, never via floating-point (NFR-06).
2. **Direction**: withdrawal vs deposit must correspond; the direction is derived from the sign of the amount.
3. **Date**: equality within a **configurable tolerance** - default ±0 days; allowed options ±1, ±2, ±3 days.
4. **Account**: the reconciliation is always scoped to exactly one target FF3 asset account.

**User-configurable parameter**: date tolerance (days). The Firefly III fetch window defaults to the tolerance — `[min date − T, max date + T]` — and can be overridden manually on the reconciliation step (PRD v0.10 amendment; formerly a fixed ±7-day search window). Rationale: padding beyond the tolerance can never yield a match — the engine pairs within T only — it merely inflates the extras group with transactions from outside the statement period.

**Pairing algorithm**: both sets are grouped by exact amount; within each group, movements and transactions are paired **1-to-1 in chronological order** inside the date tolerance. Consequences:

- Surplus movements stay `missing`; surplus transactions stay `extra`.
- Several FF3 transactions matching the same movement: the first (chronological) is `matched`; the following ones are flagged `duplicate` (RF-15).
- Movements with status `ignored` are excluded from matching entirely (RF-10).
- Bank fees / interest receive no special handling in v1: unmatched ones appear as `missing`.

## Preconditions

1. A batch with parsed movements exists (RF-02).
2. The target account is selected and its transactions fetched over the window (RF-05).
3. The batch currency matches the account currency (RF-12) - otherwise blocked.

## Steps

1. **Given** movements and FF3 transactions over the window,
2. **When** the reconciliation runs with tolerance T,
3. **Then** each movement is paired with at most one transaction with the same amount, the same direction, and a date within T days; statuses update `imported → matched/missing`, and the run is persisted with its snapshot counters, match pairs and examined transactions.

## Expected result

- Every movement leaves the run with status `matched` or `missing`.
- The three groups (plus duplicate flags) are available for display (RF-07).

## Exceptions

- **Currency mismatch** between batch and target account: the run is refused (RF-12).
- **FF3 fetch failure**: the run is aborted, statuses unchanged (RF-05 exceptions).

## Acceptance criteria

- [ ] Tolerance 0 matches only exact dates
- [ ] Tolerances ±1, ±2, ±3 respect their boundaries
- [ ] 1-to-1 chronological pairing never crosses
- [ ] A 1,000-movement statement reconciles in < 5 s (NFR-04)
- [ ] The run persists snapshots and updates statuses
- [ ] Currency mismatch is blocked
- [ ] Re-runs create a new reconciliation run
- [ ] A split FF3 transaction matches either by its total or, when no movement has that total, split by split; unmatched splits stay `extra` (TRD v0.9)

## Notes

- Each run persists its snapshot counters, match pairs and examined FF3 transactions (TRD section 2).
- The 1-to-1 pairing invariant is enforced at the storage level, not just by convention (TRD section 2).
- v0.10: the reconciliation step computes and displays the window from the tolerance (with manual override); the engine itself is unchanged.
- TRD v0.9 (2026-10-27): the statement usually shows a split FF3 transaction as one line (the total), but can list each split apart; the engine tries the total first and falls back to the splits (TRD section 6).
