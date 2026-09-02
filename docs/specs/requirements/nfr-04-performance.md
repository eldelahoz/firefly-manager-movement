# NFR-04 - Performance

| Field | Value |
| --- | --- |
| Category | Performance |
| Related | RF-06, TRD section 6 |

## Description

The reconciliation of a 1,000-movement statement completes in under 5 seconds, **excluding Firefly III API latency**. The matching engine must scale efficiently with statement size (complexity target: TRD section 6).

## Metric

Wall-clock runtime of the pure matching engine (parse to result groups), measured on the benchmark dataset.

## Threshold

< 5 s for 1,000 movements.

## Verification

- Domain-layer benchmark on the 1,000-movement reference dataset, measuring wall-clock runtime from parse to result groups.
