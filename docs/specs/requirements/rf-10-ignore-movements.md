# RF-10 - Ignore / Un-ignore Movements

| Field | Value |
| --- | --- |
| Version | v0.8 (integrates v0.6 + v0.8 amendments) |
| Priority | P1 |
| Use cases | CU-4 |
| Depends on | RF-07 (results view) |
| Related | RF-06 (engine excludes ignored), RF-08 (ignored never created) |

## Description

The user MUST be able to mark a movement as **ignored**. Ignored movements:

- are **excluded from reconciliation** - the engine loads only movements with `status != ignored` (RF-06, TRD section 6);
- are **listed in a dedicated table below the results** (RF-07 table layout), each row displaying the `Ignored` **status** and an **Un-ignore** action button;
- can be **un-ignored at any time** - un-ignore returns the movement to the group implied by the latest run's outcome;
- are **never created in Firefly III** - the apply operation only accepts `missing` movements (RF-08).

Ignoring and un-ignoring are **status changes only**: a movement is never removed from the import batch.

Typical use: internal transfers legitimately absent from one side of the comparison.

## Preconditions

1. The batch has movements (in any of `imported`, `matched`, `missing`).

## Steps

1. **Given** the results view,
2. **When** the user clicks ignore on a movement,
3. **Then** its status becomes `ignored`, it disappears from its group and appears in the ignored table with the `Ignored` status and an Un-ignore action.
4. **When** the user un-ignores it (Un-ignore button on the row),
5. **Then** the movement returns to its group (per the latest run's outcome).
6. **Given** a subsequent reconciliation run,
7. **Then** ignored movements are not matched, not counted, and their status is untouched.

## Expected result

- The ignored table always reflects the current set of ignored movements of the batch.

## Exceptions

- **Un-ignore after a re-run**: the movement rejoins the group from the latest run, not from the run at ignore time.

## Acceptance criteria

- [ ] The use case sets `status = Ignored` (ignore) and restores the latest-run status (un-ignore)
- [ ] Reconciliation excludes ignored movements and leaves their status untouched
- [ ] The ignored table renders below the results
- [ ] Each ignored row displays the `Ignored` status (v0.8)
- [ ] Each ignored row offers an Un-ignore action button (v0.8)
- [ ] Ignore is available on missing rows
- [ ] Ignoring never removes a movement from the import batch (v0.8)

## Notes

- Integrates v0.6 amendment: dedicated table + un-ignore capability.
- Integrates v0.8 amendment: `Ignored` status shown per row, Un-ignore action button per row, status-change-only semantics (the technical contract of the state change lives in the TRD).
