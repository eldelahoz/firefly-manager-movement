# RF-18 - Import Resume

| Field | Value |
| --- | --- |
| Version | v0.6 (integrates v0.5 + v0.6 amendments) |
| Priority | P1 |
| Use cases | CU-9 |
| Depends on | RF-09 (history), RF-16 (unique filenames make resume unambiguous) |
| Related | RF-02 (mapping step), RF-13 (reconcile step) |

## Description

The user MUST be able to **resume an incomplete import** from the history:

- Resuming a `parsed` batch (with movements) reopens the import wizard **at the first pending step** - the reconciliation configuration (upload and mapping are already done).
- The user can navigate **back and forward freely** between the wizard steps.
- **Wizard state resets on every opening**: a cancelled session never leaves a stale step behind (v0.5 amendment).

## Preconditions

1. A batch in `parsed` status with movements exists in the history.

## Steps

1. **Given** the imports history listing a `parsed` batch,
2. **When** the user clicks resume (play action),
3. **Then** the wizard opens directly at the reconciliation configuration step, with the import-time account pre-selected (RF-04).
4. **Given** the wizard is open,
5. **When** the user navigates back/forward between steps,
6. **Then** navigation works freely without data loss.
7. **Given** the user cancels and later reopens the wizard (new import or resume),
8. **Then** the wizard starts at the correct initial step for that context - no stale state from the cancelled session.

## Expected result

- Incomplete imports are completable without re-uploading the file.

## Exceptions

- **Probe batches** (0 movements): not offered for resume - they are auto-deleted (RF-02).

## Acceptance criteria

- [ ] `Parsed` batches are listed as resumable
- [ ] `resumeBatchId` initializes the wizard at the reconcile-config step
- [ ] Back/Next navigation works between steps
- [ ] State resets on every open

## Notes

- Integrates v0.5 (state reset) and v0.6 (first pending step + free navigation) amendments.
