# RF-12 - Currency Handling

| Field | Value |
| --- | --- |
| Version | v0.4 (integrates v0.4 amendment) |
| Priority | P1 |
| Use cases | CU-1, CU-3 |
| Depends on | RF-04 (account selection) |
| Related | RF-06 (mismatch blocks the run), RF-13 (re-run check) |

## Description

The system handles multiple currencies with a single derivation rule:

- The **statement currency is derived from the selected Firefly III account** at import time; if no currency can be derived, the system applies its `EUR` compatibility default. The currency is **never selected manually**.
- The batch currency is persisted.
- **Blocking mismatch rule**: running a reconciliation (or a re-run) against a target account whose currency differs from the batch currency MUST be blocked with a clear error before any matching happens.

## Preconditions

1. An asset account is selected at import (RF-04) or at reconciliation launch.

## Steps

1. **Given** an import targeting account A (currency X),
2. **When** the batch is created,
3. **Then** the batch is created with currency X.
4. **Given** a reconciliation launched on an account with currency Y ≠ X,
5. **When** the run starts,
6. **Then** the system blocks execution with a currency-mismatch error; nothing is persisted.

## Expected result

- Import-time account and reconcile-time account are usually the same (pre-selected, RF-04); a deliberate switch to a different currency is caught and blocked.

## Exceptions

- **Switching accounts at reconcile time**: allowed only within the same currency; otherwise blocked (the blocking warning of the original RF).
- **Default**: if no currency is provided at upload, `EUR` applies (compatibility default).

## Acceptance criteria

- [ ] Batch currency ≠ account currency blocks the run with a clear error
- [ ] The currency is derived from the selected account at import, never selected manually

## Notes

- Integrates v0.4 amendment: currency derived from the FF3 account, not manually selected.
- The batch currency is stored on the batch (TRD section 2).
