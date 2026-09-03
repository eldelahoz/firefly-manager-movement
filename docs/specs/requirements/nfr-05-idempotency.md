# NFR-05 - Idempotency of Creations

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Category | Reliability |
| Related | RF-08, RF-11, TRD section 2 (`creations`) |

## Description

Creations in Firefly III are **idempotent per movement**: a movement can never be created twice, even on retry after a partial failure.

Mechanics:

- Uniqueness of the creation record per movement is guaranteed at the storage level (TRD section 2).
- **Before each creation**, the system checks whether the movement was already created; if it was, the movement is **silently skipped** (no second creation is sent to FF3).
- The frontend displays an **indicator** for movements that have already been created.

## Metric

Number of duplicate creation attempts executed against Firefly III for an already-created movement.

## Threshold

**0** duplicate creations across any number of retries.

## Verification

- Retry scenario: a second apply on the same selection issues no new creations.
- UI: already-created movements show the indicator (Results view).
