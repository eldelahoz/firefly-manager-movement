# NFR-05 - Idempotency of Creations

| Field | Value |
| --- | --- |
| Version | v0.1 (integrates v0.1 amendment: `external_id` recovery after a partial failure) |
| Category | Reliability |
| Related | RF-08, RF-11, NFR-09, TRD section 2 (`creations`), TRD section 3 (apply), TRD section 4 (creation) |

## Description

Creations in Firefly III are **idempotent per movement**: a movement can never be created twice, even on retry after a partial failure.

Mechanics:

- Uniqueness of the creation record per movement is guaranteed at the storage level (TRD section 2).
- **Before each creation**, the system checks whether the movement was already created; if it was, the movement is **silently skipped** (no second creation is sent to FF3).
- The frontend displays an **indicator** for movements that have already been created.
- **`external_id` (v0.1)**: every transaction the app creates carries `external_id = ffm-<hash16>-<line>`, where `<hash16>` is the first 16 hex characters of the batch file hash (SHA-256) and `<line>` is the movement's line number in the file. The value depends only on the file content and the line, so it survives a reset of the app database.
- **Recovery search (v0.1)**: at the start of each apply, before any creation, the system makes one paginated Firefly III search for the prefix `ffm-<hash16>-` (`external_id_starts`), **limited to the batch's target account** (`account_id` in the query plus a client-side filter on the splits), and builds the map line → FF3 transaction id. For each selected movement without a `creations` row whose line is in the map, the system records the `creations` row with that FF3 id and reports it as already created, **without** sending a creation to FF3. The same file imported into another account is therefore **not** recognised there and creates its own transactions.
- **Conflicts (v0.1)**: a recovered transaction already matched to another movement, or a movement already matched to another transaction, is not recorded: the movement is reported `failed` with a reason asking for a new reconciliation, and nothing is created.
- **Recovery loses edits (v0.1)**: when a movement is recovered and the request carried an attachment or edits (description, category, budget, tags, notes), the report says they were not applied, because the transaction already existed.
- **Failure after creation (v0.1)**: if anything fails after Firefly III accepted the creation and before the `creations` row is stored, the batch continues; that movement is reported `failed` with a reason saying it was created in Firefly III but not recorded, and it stays `missing`. The next apply recovers it through the recovery search. If the creation request times out, the reason warns that the transaction may have been created.
- **Search failure (v0.1)**: if the recovery search fails (Firefly III down or error), the apply creates nothing and returns the error. Not creating is safer than risking a duplicate.
- **One apply at a time (v0.1)**: applies are serialized (one lock for the app), so two simultaneous applies cannot both miss the search and create the same line twice.

## Metric

Number of duplicate creation attempts executed against Firefly III for an already-created movement.

## Threshold

**0** duplicate creations across any number of retries.

## Verification

- Retry scenario: a second apply on the same selection issues no new creations.
- UI: already-created movements show the indicator (Results view).
- Acceptance criteria (v0.1):
  - [ ] Each transaction created by apply carries `external_id = ffm-<hash16>-<line>` with the first 16 hex characters of the batch file hash and the movement's line number
  - [ ] Before the first creation, apply issues one paginated search `external_id_starts:"ffm-<hash16>-"` limited to the batch's target account, however many movements are selected; none when every selected movement already has a `creations` row
  - [ ] A selected movement without a `creations` row whose line is already present in Firefly III on that account gets its `creations` row (with that FF3 id) and is reported `already_created`; no creation request is sent for it
  - [ ] A line stored on another account is not recovered: the same file imported into another account creates its own transactions
  - [ ] A recovered transaction already matched to another movement, or a movement already matched to another transaction, is reported `failed` with a reason; nothing is recorded or created
  - [ ] When a recovered movement carried an attachment or edits, the report says they were not applied
  - [ ] A failure after Firefly III created the transaction and before the `creations` row is stored reports that movement `failed` with a reason, leaves it `missing` and does not abort the other movements of the batch
  - [ ] Retrying after that failure sends no second creation for that movement: it is reported `already_created` and its `creations` row exists
  - [ ] A creation request that times out is reported `failed` with a reason warning that the transaction may exist
  - [ ] If the search fails, the apply returns an error and sends no creation request for any movement of the batch
  - [ ] Two simultaneous applies are serialized: the second starts only after the first ends

## Notes

- v0.1 amendment (`external_id`): adds the stable `external_id`, the recovery search and the failure-after-creation handling. The first reproduced scenario: Firefly III creates the transaction, then the `creations` insert, the response parsing or a timeout fails; before v0.1 no row existed and the retry created it again (reviewer finding "Ya existían antes" #1, `docs/superpowers/specs/counterparty-accounts/reports/reviewer.md`). Decisions and tasks: `docs/superpowers/specs/apply-idempotency/`.
- Scope of recovery: per account, not global, so one file reused for two accounts is never taken as already created on the wrong one. Same-account duplicates across two installations that import the same file are recognised.
- Out of scope: transactions created before v0.1 have no `external_id`, so they are not recovered by the search; the `creations` row remains their only guard. Deleting a recovered transaction in Firefly III before retrying lets the retry create it again (explicit user action, RF-15).
- RF-08 and RF-11 only reference this requirement; the `external_id` is not a user-editable field and does not replace the origin note (RF-11).
