# RF-08 - Batch Creation of Missing Transactions

| Field | Value |
| --- | --- |
| Version | v0.11 (integrates v0.6 + v0.7 + v0.8 + v0.9 + v0.10 + v0.11 amendments) |
| Priority | P0 |
| Use cases | CU-5 |
| Depends on | RF-07 (missing group available), RF-11 (origin trace) |
| Related | RF-10 (ignored never created), NFR-05 (idempotency), NFR-09 (partial failure report) |

## Description

The user selects missing movements (individual checkboxes + "select all") and has **two actions** available:

1. **Personalize transactions** - opens the personalization editor for the selection: the **edit sheet**, in single mode for one movement (with Previous/Next between rows) or in bulk mode when several movements are selected. The editor is pre-populated with the existing Firefly III categories, budgets and tags. Available fields:

| Field | Status |
| --- | --- |
| Counterparty (v0.10) | One of: an existing FF3 account (expense, revenue, liability or own asset account), a **new** expense/revenue account typed by name, or the explicit **Cash / no counterparty** option. Picked from a searchable list grouped by type and filtered by direction (see the rules table) |
| Description | Editable - pre-filled from the statement |
| Category | Optional - picked from existing FF3 categories |
| Budget | Optional - picked from existing FF3 budgets |
| Tags | Optional - picked from existing FF3 tags; pre-filled with the default creation tag configured in settings (RF-19) - the user's selection replaces it |
| Attachment (file) | Optional - uploaded and linked to the created transaction |
| Notes | Optional |

**Bulk semantics**: in the edit sheet's bulk mode, **only the fields the user fills are applied** to every selected movement; fields left empty leave each movement's own values untouched.

**Account semantics (v0.8, reworded v0.10)**: description, date and amount are required and always present from the statement (validated pre-creation). The reconciled target account (call it A) is always an asset account and always the statement side - the **source** when money leaves (negative amount), the **destination** when money enters (positive amount); it is pre-filled and not editable here. The other side is the counterparty (below).

**UI labels (v0.10)**: the counterparty field is labelled "Destination account" when money leaves and "Source account" when money enters. Specs and API keep calling it the counterparty.

1. **Create in Firefly III** - opens a **confirmation modal** summarizing what will be created (per movement: description, amount and the personalizations applied, including tags); on confirmation the batch creation runs.

**Two paths**: the user may create the movements as-is (statement data + optional counterparty account + default creation tag) or personalize them first - individually or in bulk - and then create. Both paths end with the confirmation modal.

**Validation**: creation is refused for a movement whose amount is 0 or whose date is invalid (both are already prevented at parse level and re-checked before creation).

**Creation**: the system creates each transaction in Firefly III with its type, amount, date, description, source/destination accounts and the origin trace (RF-11: a tag plus optional notes identifying the source file and batch).

**Movement classification (v0.10, replaces the v0.8 rule "transfers never generated")**: the transaction type is derived on the backend from the **amount sign** (negative = money out, positive = money in) and the **counterparty**, following Firefly III's own `account_to_transaction` rules. A is the batch's asset account.

| # | Counterparty | Money out (negative) | Money in (positive) |
| --- | --- | --- | --- |
| 1 | Existing expense account | `withdrawal` A → expense | rejected |
| 2 | Existing revenue account | rejected | `deposit` revenue → A |
| 3 | Liability (loan, debt, mortgage) | `withdrawal` A → liability | `deposit` liability → A |
| 4 | Own asset account (not A, same currency; includes `ccAsset` credit cards) | `transfer` A → asset | `transfer` asset → A |
| 5 | New name (`counterparty_name`) | `withdrawal` A → new expense account (`destination_name`) | `deposit` new revenue account (`source_name`) → A |
| 6 | None (explicit Cash / no counterparty) | `withdrawal` A → Cash | `deposit` Cash → A |

**Rejected combinations** (never sent to Firefly III): the counterparty is A itself; an expense account on money in; a revenue account on money out; an own asset account in a currency other than A's (a transfer would need a foreign amount, out of scope); a new name together with an id; a counterparty id that is not a known cached account.

**Rejection behavior**: a rejected combination does **not** fail the batch. That movement's report is `failed` with a `reason` that names the cause, no `creations` row is written and the movement stays `missing` so the user can fix it and retry. Each report carries `transaction_type` (`withdrawal`, `deposit` or `transfer`; null when failed).

**Counterparty input**: `counterparty_ff3_id` (existing account) or `counterparty_name` (create by name); both absent and `counterparty_cash` not true = **no counterparty chosen** (v0.11): the transaction is created against Cash but the choice is not learned; `counterparty_cash: true` (optional boolean, v0.11) = explicit Cash, learned; an id or a name together with `counterparty_cash: true`, or both id and name, = rejected. Creating accounts by name applies only to expense (money out) and revenue (money in) accounts; assets and liabilities cannot be created from the app.

**Counterparty suggestion (v0.11)**: for each `missing` movement the app **suggests** the counterparty so the user does not pick the same account every month. The app is generic: there are no rules for a particular bank, language or user, and nothing is created on its own.

| Aspect | Rule |
| --- | --- |
| Sources and precedence | 1. **Learned** (`learned`): what the user chose before in this app. 2. **History** (`history`): what Firefly III shows for past transactions. History is consulted only when nothing is learned; learned always wins |
| Normalization (both sources) | One generic function applied to the movement description and to the stored descriptions: lowercase; strip accents (composed or decomposed) and fold Latin letters without a decomposition (`ß`→`ss`, `ø`→`o`, `æ`→`ae`, `ł`→`l`); keep letters of other scripts; replace punctuation with spaces; drop digit runs of 4 or more characters (references, dates, card numbers); collapse spaces; trim. Example: `PAGO PSE Acme Store S.A. 20261117 REF 99812` → `pago pse acme store s a ref`. No word lists, no bank prefixes, no fuzzy or partial matching: the normalized descriptions must be equal. A movement whose normalized description is empty gets no suggestion |
| Scope | The batch's account (A) **and** the direction (money out / money in). A choice learned for money out is never suggested for money in, nor from another account |
| Learned | Every movement created through the app teaches `normalized description → counterparty` for A and the direction, with the counterparty actually used: an existing account, a new name (and, when Firefly III returned it, the id of the account it created) or Cash. The latest choice replaces the previous one; a use count is kept. A movement created with no counterparty chosen (suggestion cleared, no Cash) teaches nothing |
| History | Firefly III transactions of A over a lookback period (**180 days by default**, ending at the statement window's end), read from the side opposite to A. The suggestion is the counterparty used most often for that normalized description; ties go to the most recent transaction. The Cash account is ignored, and so is A itself. **Known risk**: descriptions that differ only by long numbers (for example transfers to different people, whose account numbers are dropped) share one suggestion; the row and the confirmation dialog show the account before anything is created |
| Suggestion only | The single sheet is pre-filled with the suggestion and shows a mark: "Suggested (learned)" or "Suggested (Firefly III history)". The user can change or clear it (clearing leaves no counterparty chosen: the movement is created against Cash but the choice is not learned; the explicit Cash option sends `counterparty_cash: true` and is learned). The bulk sheet is not pre-filled: it shows "Leave unchanged", each selected row keeps its own suggestion, and a counterparty filled in bulk replaces it for every selected row (fill-only semantics). Creation still requires the explicit confirmation; a suggestion is a counterparty like any other: the rules table and the rejections apply to it |

**Per-movement processing**: an FF3 error on one movement MUST NOT abort the batch; the run continues and ends with a per-movement report (success/failure/reason) (NFR-09).

**Idempotency**: before each creation the system checks whether the movement was already created; already-created movements are skipped silently (NFR-05). Each created transaction also carries a stable `external_id` that lets a retry recover a creation that Firefly III accepted but the app did not record (NFR-05 v0.1). Only movements in `missing` status are accepted.

**After success**: the movement moves to `created`; a confirmation re-reconciliation moves `created → matched` without losing provenance (status semantics, RF-09).

**No automatic mode**: creation happens exclusively on explicit user selection and confirmation (objective O4).

## Preconditions

1. The batch is reconciled and has movements in `missing` status (RF-07).
2. The user has selected at least one movement.

## Steps

1. **Given** the results view with missing movements,
2. **When** the user selects movements,
3. **Then** the "personalize" and "create" actions become available.
4. **When** the user personalizes the selection - in the edit sheet, single or bulk (bulk applies only filled fields),
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

- [ ] (v0.10, table row 1) Money out + existing expense account creates a `withdrawal` with source = A and destination = that account; money in + expense account is rejected
- [ ] (v0.10, row 2) Money in + existing revenue account creates a `deposit` with source = that account and destination = A; money out + revenue account is rejected
- [ ] (v0.10, row 3) A liability counterparty creates a `withdrawal` A → liability on money out and a `deposit` liability → A on money in
- [ ] (v0.10, row 4) An own asset account (same currency, not A, including `ccAsset`) creates a `transfer` A → asset on money out and asset → A on money in; the payload carries both account ids and the absolute amount
- [ ] (v0.10, row 4) An own asset account in a currency other than A's is rejected before calling Firefly III
- [ ] (v0.10, row 5) `counterparty_name` creates a `withdrawal` with `destination_name` (money out) or a `deposit` with `source_name` (money in); Firefly III creates the expense/revenue account; no account id is sent for that side
- [ ] (v0.10, row 6) With `counterparty_cash: true` (and no id or name), a `withdrawal` to Cash (money out) or a `deposit` from Cash (money in) is created; this is an explicit choice, shown as "Cash / no counterparty"
- [ ] (v0.10) The counterparty equal to A, and `counterparty_ff3_id` together with `counterparty_name`, are rejected before calling Firefly III
- [ ] (v0.10) A rejected movement is reported `failed` with a `reason` naming the cause; the other movements of the batch are still created; no `creations` row exists for it and it stays `missing`
- [ ] (v0.10) Each report carries `transaction_type` (`withdrawal`, `deposit`, `transfer`), null when failed; the type is never taken from the client
- [ ] (v0.10) Retrying a rejected movement with a valid counterparty creates it; retrying created movements creates nothing twice
- [ ] (v0.11) The normalization is a pure function: `PAGO PSE Acme Store S.A. 20261117 REF 99812` → `pago pse acme store s a ref`; accents are stripped (`Café` → `cafe`), digit runs of 4 or more characters are dropped and runs of 3 or fewer are kept (`Tienda 24` → `tienda 24`)
- [ ] (v0.11) After a movement is created with an existing account, a new name or Cash, a movement of a later batch on the same account and direction with the same normalized description gets that choice as `learned` suggestion
- [ ] (v0.11) The latest choice replaces the earlier one for the same account, direction and normalized description; the same description on another account or in the other direction gets no learned suggestion
- [ ] (v0.11) With a learned choice and a different one in Firefly III history, the suggestion is the learned one (`source: "learned"`)
- [ ] (v0.11) With nothing learned, the suggestion is the counterparty most used in the account's Firefly III transactions of the last 180 days for the same normalized description (`source: "history"`); the Cash account and A itself are never suggested from history
- [ ] (v0.11) With nothing learned and no history match, `suggested_counterparty` is `null` and the sheet comes without a counterparty
- [ ] (v0.11) A suggestion always returns the account id together with its name; if that account no longer exists, applying it is rejected for that movement with a `reason` (the batch continues)
- [ ] (v0.11) UI: the single sheet pre-fills the counterparty from the suggestion with the mark "Suggested (learned)" or "Suggested (Firefly III history)" (en/fr/es); the bulk sheet is not pre-filled: its counterparty shows "Leave unchanged" and each selected row keeps its own suggestion unless the user fills the bulk counterparty, which then replaces it for every selected row; the Missing row shows the suggested account; the confirmation dialog shows it like any other counterparty
- [ ] (v0.11) UI: the user can change the suggested counterparty or clear it; once changed or cleared the mark disappears, and clearing sends no counterparty and no `counterparty_cash` (the movement is created against Cash but not learned), while the explicit Cash option sends `counterparty_cash: true` and is learned
- [ ] (v0.11) A movement created with no counterparty chosen does not change the learned memory; one created with explicit Cash (`counterparty_cash: true`) does
- [ ] (v0.11) A failed history read during reconciliation keeps the history suggestions from the previous run
- [ ] (v0.11) Descriptions that differ only by digit runs of 4 or more characters share one suggestion; the row and the confirmation dialog show the suggested account before creation
- [ ] (v0.11) No transaction is created, and no account is created, until the user confirms the creation; reconciling or opening the sheet only reads
- [ ] (v0.11) Learned suggestions are resolved when the results are read, so a batch reconciled before this version can show a `learned` suggestion when the memory already has its description; its `history` suggestions are `null` until it is reconciled again
- [ ] Creation includes the origin tag, category and notes
- [ ] Partial failure continues the batch with a per-movement report
- [ ] Retry creates nothing twice
- [ ] Non-`missing` movements are rejected
- [ ] The system applies the user's selection in a single batch operation
- [ ] UI: selection with individual checkboxes and select-all
- [ ] UI: "personalize transactions" action for single and multi-selection
- [ ] UI: edit sheet in bulk mode applies only filled fields; empty fields leave per-movement values untouched
- [ ] UI: tags picker pre-filled with the default creation tag (RF-19); the user's selection replaces it
- [ ] UI: "create in Firefly III" action opens a confirmation modal summarizing the movements to be created
- [ ] UI: edit sheet (single mode) edits description, category, budget, tags, notes, attachment
- [ ] UI (v0.10): the counterparty picker is a searchable list grouped by type. Money out: Expense accounts, Liabilities, Own accounts (credit cards badged), then "Create «X» as expense account" and "Cash / no counterparty". Money in: the same with Revenue accounts and "Create «X» as revenue account". A is never listed
- [ ] UI (v0.10): the field label is "Destination account" for money out and "Source account" for money in (en/fr/es)
- [ ] UI (v0.10): a line under the picker states the resulting type (withdrawal, deposit or transfer) using the table rules; the confirmation modal shows the type and the counterparty of each movement
- [ ] UI (v0.10): the bulk sheet uses the same picker when all selected movements share one direction; a mixed selection offers none
- [ ] UI (v0.10): rows rejected by the backend show the `failed` pill with the reason; the type of created rows comes from `transaction_type`

## Notes

- Integrates v0.6 amendment: editable description/category/budget/tags/attachment/notes with FF3-sourced pickers.
- Integrates v0.7 amendments: two-action flow (personalize / create) with confirmation modal; bulk personalization with fill-only semantics; tags pre-filled with the default creation tag (RF-19).
- Integrates v0.8 amendment: movement classification by amount sign - expenses and incomes only, transfers never generated; the counterparty account is optional (when omitted, Firefly III applies its default handling for the movement). **Superseded by v0.10** (transfers allowed to own accounts; Cash is an explicit option).
- Integrates v0.9 amendment (wording only, no functional change): "inline editing" and "bulk panel" become one **edit sheet** with single and bulk modes; fill-only semantics, fields and the two-action flow are unchanged.
- Integrates v0.10 amendment (counterparty accounts, phase A): replaces the v0.8 rule "transfers never generated" with the rules table (derived from Firefly III `account_to_transaction`, verified on 6.6.6); adds liabilities, own accounts as transfers (including `ccAsset`), creation of expense/revenue accounts by name and the explicit Cash option; the counterparty is no longer "optional with Firefly III's default" but an explicit choice (Cash when none). Out of scope: transfers across currencies, editing existing FF3 transactions, creating assets or liabilities. (Remembering the counterparty and suggesting it was out of scope here and is added by v0.11.)
- Integrates v0.11 amendment (counterparty suggestions, phase B): suggests the counterparty of each missing movement, first from what the user chose before in the app and then from Firefly III history; generic normalization, scope by batch account and direction, 180-day history lookback; suggestion only, marked, changeable and clearable. Out of scope: fuzzy or partial matching, suggesting category, budget or tags, editing or deleting learned entries in Settings, Firefly III rules.
- The zero-amount guard is also enforced at the storage level (TRD section 2).
- The already-created check is backed by a storage-level uniqueness guarantee (TRD section 2).
