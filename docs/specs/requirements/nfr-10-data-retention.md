# NFR-10 - Data Retention

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Category | Data management |
| Related | RF-09, RF-15, RF-17 |

## Description

- Batches and movements are retained **indefinitely**, until the user deletes them - and deletion is only possible for `parsed` batches via the explicit RF-17 flow.
- `reconciled`/`applied` batches and their run history are permanent (RF-09).
- Audit snapshots of deleted transactions are retained **indefinitely**.
- The system runs **no automatic cleanup**: no TTL, no scheduled purge jobs, no size-based eviction.

## Metric

Existence of automatic data-removal mechanisms; retention of audit snapshots over time.

## Threshold

- **0** automatic deletion jobs/schedules.
- Audit snapshots: unbounded retention.

## Verification

- Codebase review: no scheduled or background cleanup affecting stored batches, movements, reconciliation history or audit snapshots.
- The only deletion paths are RF-17 (user-confirmed, `parsed` only) and RF-02's probe cleanup (headers-only batches).
