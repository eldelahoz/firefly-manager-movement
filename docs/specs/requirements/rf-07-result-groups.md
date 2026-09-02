# RF-07 - Result Groups

| Field | Value |
| --- | --- |
| Priority | P0 |
| Use cases | CU-4 |
| Depends on | RF-06 (a reconciliation run exists) |
| Related | RF-08 (creation), RF-10 (ignored section), RF-15 (duplicates in extras) |

## Description

Results MUST be presented in **three groups** with visual highlighting of the gaps:

| Group | Content | Action |
| --- | --- | --- |
| **Matched** (`matched`) | Movements present on both sides | None (read-only) |
| **Missing** (`missing`) | In the statement, absent from FF3 | Create in FF3 - single or batch (RF-08) |
| **Extras** (`extra`) | In FF3, absent from the statement | Informative in v1; deletion possible when flagged `duplicate` (RF-15) |

A fourth, dedicated section lists **ignored** movements (RF-10). Duplicate-flagged transactions carry a visible badge in the extras group (RF-15). The results view MUST remain responsive with large lists (virtualized rendering).

## Preconditions

1. At least one reconciliation run has completed on the batch (RF-06).

## Steps

1. **Given** a reconciled batch,
2. **When** the user opens the results view,
3. **Then** the system displays the three groups (with per-group counts) plus the ignored section, each row showing date, amount and description, with visual highlighting of gaps.

## Expected result

- The latest run's results - the three groups plus the ignored movements - are available for display at any time.
- The UI renders the groups in separate tabs/sections with badges and highlighting.

## Exceptions

- **Empty group**: the section renders with an explicit empty state, never omitted silently.

## Acceptance criteria

- [ ] The engine produces `matches`, `missing`, `extra` and `duplicates`
- [ ] Run counts (matched/missing/extra/duplicate) are persisted
- [ ] The latest run's results (3 groups + ignored) are available at any time
- [ ] The results page renders three tabs/sections
- [ ] Rows render with checkboxes, editable fields and duplicate flags
- [ ] Large lists stay performant
