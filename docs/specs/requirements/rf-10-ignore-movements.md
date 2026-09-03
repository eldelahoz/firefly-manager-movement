# RF-10 - Ignore / Un-ignore Movements

| Field | Value |
| --- | --- |
| Version | v0.6 (integrates v0.6 amendment) |
| Priority | P1 |
| Use cases | CU-4 |
| Depends on | RF-07 (results view) |
| Related | RF-06 (engine excludes ignored), RF-08 (ignored never created) |

## Description

The user MUST be able to mark a movement as **ignored**. Ignored movements:

- are **excluded from reconciliation** - the engine loads only movements with `status != ignored` (RF-06, TRD section 6);
- are **visible in a dedicated section** of the results view, below the three groups;
- can be **un-ignored at any time** - un-ignore returns the movement to the group implied by the latest run's outcome;
- are **never created in Firefly III** - the apply operation only accepts `missing` movements (RF-08).

Typical use: internal transfers legitimately absent from one side of the comparison.

## Preconditions

1. The batch has movements (in any of `imported`, `matched`, `missing`).

## Steps

1. **Given** the results view,
2. **When** the user clicks ignore on a movement,
3. **Then** its status becomes `ignored`, it disappears from its group and appears in the ignored section.
4. **When** the user un-ignores it,
5. **Then** the movement returns to its group (per the latest run's outcome).
6. **Given** a subsequent reconciliation run,
7. **Then** ignored movements are not matched, not counted, and their status is untouched.

## Expected result

- The ignored section always reflects the current set of ignored movements of the batch.

## Exceptions

- **Un-ignore after a re-run**: the movement rejoins the group from the latest run, not from the run at ignore time.

## Acceptance criteria

- [ ] The use case toggles `status = Ignored`
- [ ] Reconciliation excludes ignored movements and leaves their status untouched
- [ ] The ignored section renders with an un-ignore action
- [ ] Ignore is available on missing rows

## Notes

- Integrates v0.6 amendment: dedicated section + un-ignore capability.
