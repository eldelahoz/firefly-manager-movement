# NFR-06 - Amount Integrity

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Category | Data integrity |
| Related | RF-06, TRD section 2 |

## Description

Monetary amounts are handled as **signed integer cent values** (or exact decimals where unavoidable) and **never as floating-point numbers**, in storage, computation and comparison alike. The direction (withdrawal/deposit) is derived from the sign.

## Metric

Occurrences of floating-point types in amount-handling code paths (storage, computation, comparison).

## Threshold

**0** float usages for monetary values (storage, computation, comparison).

## Verification

- Code review: all amount computations and comparisons use exact integer values.
- Schema review: amounts are stored as exact integers, never floats (TRD section 2).
