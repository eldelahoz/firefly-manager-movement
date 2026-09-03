# NFR-09 - Error Handling

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Category | UX / Reliability |
| Related | RF-01, RF-04, RF-05, RF-08, RF-14, RF-15, RF-19 |

## Description

1. **Firefly III unavailable**: the system displays a clear error message identifying the connection problem and offers a retry action. No silent failure, no empty state masquerading as "no data".
2. **Progress indication**: long operations - reconciliation runs and batch creations - display a progress indicator while running.
3. **Partial failure reporting**: when a batch creation succeeds only partially, the system displays a detailed per-movement report (success / failure / reason); the batch continues past individual failures (RF-08).

## Metric

Coverage of error scenarios in the frontend test suite (FF3 errors, timeouts, partial batch failures); presence of progress indicators on long operations.

## Threshold

- **100%** of FF3-calling features surface errors with a retry affordance.
- **2/2** long operations (reconcile, apply) show a progress indicator.

## Verification

- Mocked error scenarios in the frontend test suite (parse errors banner, failed connectivity check, results/apply failures).
- Partial-failure scenario: the batch continues with a per-movement report including reasons.
