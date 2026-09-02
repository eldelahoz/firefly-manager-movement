# NFR-07 - Observability

| Field | Value |
| --- | --- |
| Category | Observability |
| Related | RF-08, RF-15, TRD section 7 |

## Description

Imports, reconciliations and creations produce **structured JSON logs** with the following fields:

| Field | Content |
| --- | --- |
| `timestamp` | ISO-8601 event time |
| `level` | `info` / `warn` / `error` |
| `message` | Human-readable event description |
| `context` | Identifiers: `batch_id`, `movement_id`, `reconciliation_id`, `ff3_id` - as applicable |

**Level semantics**:

- `info`: normal operations (batch created, reconciliation run, movement created, transaction deleted).
- `warn`: recoverable errors (invalid CSV line kept, single-movement creation failure within a continuing batch).
- `error`: critical failures (FF3 API unreachable, database error).

Logs are written to stdout/stderr and are therefore consumable via `docker logs` - no external log storage in v1.

## Metric

Share of log lines emitted as valid structured JSON with the required fields.

## Threshold

**100%** of backend log lines are structured JSON with `timestamp`, `level`, `message` and applicable context ids.

## Verification

- Capture backend stdout/stderr during integration tests; assert every line parses as JSON with the required fields and correct level usage.
