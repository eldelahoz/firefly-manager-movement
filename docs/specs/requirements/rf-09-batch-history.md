# RF-09 - Batch Immutability and History

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Priority | P1 |
| Use cases | CU-6 |
| Depends on | RF-01 (batches exist) |
| Related | RF-13 (run history), RF-17 (deletion of parsed batches), RF-18 (resume) |

## Description

Each import batch MUST be immutable and viewable a posteriori:

- A batch follows the lifecycle `parsed → reconciled → applied` (PRD section 4); its movements and metadata are never edited in place - only statuses evolve through reconciliation runs and user actions.
- Every reconciliation run is historized with frozen snapshot counters (RF-13).
- The history view lists past batches with their status and offers resume/delete actions per status (RF-17, RF-18).
- Deletion is only possible for `parsed` batches (RF-17); `reconciled`/`applied` batches are immutable and permanent.

## Preconditions

1. At least one import exists.

## Steps

1. **Given** the user opens the imports history,
2. **When** the list loads,
3. **Then** every batch appears with filename, status, dates and counts, and per-status actions (resume/delete for `parsed` - RF-17/RF-18).

## Expected result

- The complete information of every batch (metadata, statuses, counts) remains available at any time after import.

## Exceptions

- None specific: the history is always available; a missing batch id returns "not found".

## Acceptance criteria

- [ ] All batches are listed with status
- [ ] A batch is persisted with full metadata in `parsed` status on upload
- [ ] Each batch's full details are viewable individually
- [ ] The history UI shows status badges and resume/delete actions

## Notes

- Each reconciliation run is historized as its own record with frozen snapshot counters (TRD section 2).
