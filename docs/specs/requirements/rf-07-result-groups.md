# RF-07 - Result Groups

| Field | Value |
| --- | --- |
| Version | v0.8 (integrates v0.8 amendment) |
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

## Table layout (v0.8)

Each group renders as a **table with column headers**. Columns per group:

| Group | Columns |
| --- | --- |
| Matched | Date · Description (statement) · Description (Firefly III) · Amount · Status |
| Missing | Selection · Date · Description · Amount · Status · Actions (Personalize, Ignore) |
| Extras | Date · Description · Amount · Status (duplicate badge, RF-15) · Actions (RF-15) |
| Ignored | Date · Description · Amount · Status (`Ignored`) · Actions (Un-ignore) |

The Status column always reflects the movement's real state, including the post-creation report pills (`created` / `failed`) on missing rows (RF-08).

## Preconditions

1. At least one reconciliation run has completed on the batch (RF-06).

## Steps

1. **Given** a reconciled batch,
2. **When** the user opens the results view,
3. **Then** the system displays the three groups (with per-group counts) plus the ignored table, each rendered with the columns defined in the table layout (v0.8) and visual highlighting of gaps.

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
- [ ] Each group renders as a table with column headers per the table layout (v0.8)
- [ ] The Status column reflects the movement's real state, including post-apply report pills
- [ ] Large lists stay performant
