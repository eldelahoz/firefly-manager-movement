# RF-13 - Reconciliation Re-run

| Field | Value |
| --- | --- |
| Version | v0.10 (integrates PRD v0.10 amendment) |
| Priority | P1 |
| Use cases | CU-3 |
| Depends on | RF-06 (engine), RF-09 (batch persists) |
| Related | RF-12 (currency check on new account), NFR-05 (created movements stay created) |

## Description

The user MUST be able to **re-run a reconciliation on an existing batch** with **modified parameters**:

- date tolerance (±0..±3 days),
- search window (defaults to the tolerance, editable via the manual override; re-runs recompute it from the new tolerance unless overridden),
- target account (subject to the RF-12 currency rule).

**Semantics of a re-run**:

- **A new reconciliation run is created**: each run records fresh snapshot counters - the run history is preserved and viewable a posteriori (RF-09).
- **Movement statuses are overwritten** to reflect the latest run (`imported → matched/missing`; `created → matched` on confirmation runs; `ignored` untouched - RF-10).
- Previous runs' matches and examined transactions remain persisted (per-run historization); the results view shows the latest run by default.

## Preconditions

1. The batch exists and already has at least one reconciliation run (re-running a `parsed` batch is simply a first run - RF-06).

## Steps

1. **Given** a reconciled batch,
2. **When** the user opens the reconciliation configuration ("re-run") and adjusts tolerance/window/account,
3. **Then** a new run executes with the new parameters, the run history is appended (RF-09), and movement statuses reflect the new outcome.

## Expected result

- Re-running is always possible on an existing batch; it never duplicates movements or corrupts prior runs.

## Exceptions

- **Currency mismatch on a new account**: blocked (RF-12).
- **FF3 unavailable**: run aborted, previous results untouched (NFR-09).
- **Created movements**: provenance survives even if a new run marks them `matched`.

## Acceptance criteria

- [ ] A second run creates a new run record and updates statuses
- [ ] Results can be fetched for the latest run
- [ ] A reconciliation can be re-run any number of times on the same batch
- [ ] The "re-run" action is offered on already-reconciled batches

## Notes

- Each run is stored as its own record with fresh snapshot counters (TRD section 2).
- Created-movement provenance survives re-runs (see NFR-05).
