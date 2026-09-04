# RF-08 - Batch Creation of Missing Transactions

| Field | Value |
| --- | --- |
| Version | v0.8 (integrates v0.6 + v0.7 + v0.8 amendments) |
| Priority | P0 |
| Use cases | CU-5 |
| Depends on | RF-07 (missing group available), RF-11 (origin trace) |
| Related | RF-10 (ignored never created), NFR-05 (idempotency), NFR-09 (partial failure report) |

## Description

The user selects missing movements (individual checkboxes + "select all") and has **two actions** available:

1. **Personalize transactions** - opens the personalization editor for the selection: inline editing for a single movement, or a **bulk panel** when several movements are selected. The editor is pre-populated with the existing Firefly III categories, budgets and tags. Available fields:

| Field | Status |
| --- | --- |
| Counterparty account (expense/income account, or asset account depending on direction) | **Optional** - picked from existing FF3 accounts, listed by direction; when omitted, Firefly III applies its default handling for the movement |
| Description | Editable - pre-filled from the statement |
| Category | Optional - picked from existing FF3 categories |
| Budget | Optional - picked from existing FF3 budgets |
| Tags | Optional - picked from existing FF3 tags; pre-filled with the default creation tag configured in settings (RF-19) - the user's selection replaces it |
| Attachment (file) | Optional - uploaded and linked to the created transaction |
| Notes | Optional |

**Bulk semantics**: in the bulk panel, **only the fields the user fills are applied** to every selected movement; fields left empty leave each movement's own values untouched.

**Account semantics (v0.8)**: description, date and amount are required and always present from the statement (validated pre-creation). The reconciled target account is always the opposite side - the **source** for an expense (negative amount), the **destination** for an income (positive amount); it is pre-filled and not editable here. The counterparty account is optional: when omitted, the field is simply not sent and Firefly III applies its default handling for the movement.

2. **Create in Firefly III** - opens a **confirmation modal** summarizing what will be created (per movement: description, amount and the personalizations applied, including tags); on confirmation the batch creation runs.

**Two paths**: the user may create the movements as-is (statement data + optional counterparty account + default creation tag) or personalize them first - individually or in bulk - and then create. Both paths end with the confirmation modal.

**Validation**: creation is refused for a movement whose amount is 0 or whose date is invalid (both are already prevented at parse level and re-checked before creation).

**Creation**: the system creates each transaction in Firefly III with its type, amount, date, description, source/destination accounts and the origin trace (RF-11: a tag plus optional notes identifying the source file and batch).

**Movement classification (v0.8)**: the amount's sign determines the kind of transaction created - a **negative amount is an expense** (money leaves the account), a **positive amount is an income** (money enters the account). The reconciler only creates expenses and incomes; it never creates internal transfers, since a bank statement movement is always money in or out of the reconciled account.

**Per-movement processing**: an FF3 error on one movement MUST NOT abort the batch; the run continues and ends with a per-movement report (success/failure/reason) (NFR-09).

**Idempotency**: before each creation the system checks whether the movement was already created; already-created movements are skipped silently (NFR-05). Only movements in `missing` status are accepted.

**After success**: the movement moves to `created`; a confirmation re-reconciliation moves `created → matched` without losing provenance (status semantics, RF-09).

**No automatic mode**: creation happens exclusively on explicit user selection and confirmation (objective O4).

## Preconditions

1. The batch is reconciled and has movements in `missing` status (RF-07).
2. The user has selected at least one movement.

## Steps

1. **Given** the results view with missing movements,
2. **When** the user selects movements,
3. **Then** the "personalize" and "create" actions become available.
4. **When** the user personalizes the selection - inline or via the bulk panel (only filled fields applied),
5. **Then** each movement keeps its personalized values, visible in the results table.
6. **When** the user triggers the creation and confirms in the modal,
7. **Then** the system creates each selected movement in FF3 (skipping already-created ones), applies the origin trace, and returns a per-movement report.

## Expected result

- Selected movements transition to `created` (then `matched` after confirmation re-reconciliation).
- Non-selected movements are untouched; `ignored` movements are never created (RF-10).

## Exceptions

- **Movement not in `missing` status**: refused.
- **FF3 error on some movements**: the batch continues; the report lists failures with reasons (NFR-09).
- **Retry after partial failure**: already-created movements are not re-created (NFR-05).

## Acceptance criteria

- [ ] Negative amounts create expenses and positive amounts create incomes; internal transfers are never created
- [ ] Creation works without a counterparty account (optional field)
- [ ] Creation includes the origin tag, category and notes
- [ ] Partial failure continues the batch with a per-movement report
- [ ] Retry creates nothing twice
- [ ] Non-`missing` movements are rejected
- [ ] The system applies the user's selection in a single batch operation
- [ ] UI: selection with individual checkboxes and select-all
- [ ] UI: "personalize transactions" action for single and multi-selection
- [ ] UI: bulk panel applies only filled fields; empty fields leave per-movement values untouched
- [ ] UI: tags picker pre-filled with the default creation tag (RF-19); the user's selection replaces it
- [ ] UI: "create in Firefly III" action opens a confirmation modal summarizing the movements to be created
- [ ] UI: inline edit of description, category, budget, tags, notes, attachment
- [ ] UI: counterparty picker lists FF3 accounts by direction (expense/income/asset)

## Notes

- Integrates v0.6 amendment: editable description/category/budget/tags/attachment/notes with FF3-sourced pickers.
- Integrates v0.7 amendments: two-action flow (personalize / create) with confirmation modal; bulk personalization with fill-only semantics; tags pre-filled with the default creation tag (RF-19).
- Integrates v0.8 amendment: movement classification by amount sign - expenses and incomes only, transfers never generated; the counterparty account is optional (when omitted, Firefly III applies its default handling for the movement).
- The zero-amount guard is also enforced at the storage level (TRD section 2).
- The already-created check is backed by a storage-level uniqueness guarantee (TRD section 2).
