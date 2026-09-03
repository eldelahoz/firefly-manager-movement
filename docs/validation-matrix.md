# RF Validation Matrix — firefly-manager-movement

Maps each **Functional Requirement (RF-01 .. RF-19)** to the tests that verify it.

> **Legend**
> - **Domain** → `backend/domain/tests/*.rs` (pure logic, no DB)
> - **App** → `backend/application/tests/*.rs` (use cases with fake repos)
> - **Infra** → `backend/infrastructure/tests/*.rs` (Diesel repos, Firefly client)
> - **Pres** → `backend/presentation/tests/*.rs` (axum HTTP handlers)
> - **FE** → `frontend/src/**/*.test.tsx` (React + Vitest + MSW)

> **Status** reflects the real implementation state against the *current* spec (including amendments), not just test existence:
> - `implemented` — matches the current spec
> - `partial (spec v0.X)` — base works; the listed amendment deltas are pending
> - `pending (spec v0.X)` — the amended functionality is not implemented
>
> Spec versions are declared in the `Version` field of each requirement doc (`docs/specs/requirements/`): `v0.X (integrates ...)` for amended requirements, `Original (2026-08-14) — unamended` otherwise.

---

## RF-01  CSV upload (max 10 MB); XLSX → CSV by user

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `upload_import.rs::upload_import_parses_valid_csv` | Valid CSV → batch + movements created |
| **App** | `upload_import.rs::upload_import_keeps_invalid_lines` | Invalid lines reported, valid kept (also RF-03) |
| **App** | `upload_import.rs::upload_import_computes_file_hash` | SHA-256 hash computed for dedup |
| **App** | `upload_import.rs::upload_import_rejects_duplicate_filename` | 409 Conflict before any data stored (also RF-16) |
| **App** | `csv.rs` (multiple) | CSV parsing, delimiter detection, encoding |
| **Infra** | `csv_decoder.rs` | Low-level CSV decode, delimiter sniffing |
| **FE** | `ImportWizard.test.tsx::oversize file shows inline error` | Client-side 10 MB guard, no upload attempted |
| **FE** | `ImportWizard.test.tsx::drop fires upload` | Drag-drop parity with picker |
| **FE** | `ImportWizard.test.tsx::parse error shows banner` | Server error surfaced, dropzone reset |

**No XLSX tests** — by design (RF-01: user converts to CSV).

---

## RF-02  CSV parsing with manual mapping; memorize by header signature

**Status:** partial (spec v0.7) · **Last verified:** 2026-09-03
**Pending (v0.7):** optional origin-account column in the mapping and parser (feeds RF-04 auto-suggestion). The v0.5 deltas (explanatory message on manual mapping, automatic probe deletion) are implemented and tested.

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `upload_import.rs::upload_import_no_mapping_returns_headers_only` | Unknown headers → batch created, 0 movements, headers returned |
| **App** | `upload_import.rs::upload_import_uses_provided_mapping` | Explicit `ColumnMapping` used |
| **App** | `upload_import.rs::upload_import_uses_saved_mapping_by_signature` | Header signature lookup → auto-apply saved mapping |
| **App** | `save_csv_mapping.rs` | `PUT /mapping` persists mapping by signature |
| **App** | `csv.rs` | Auto-detection for known Spanish headers (`Fecha;Valor` etc.) |
| **Infra** | `csv_mapping_repo.rs` | `CsvMappingRepository` CRUD + `by_signature` |
| **FE** | `ImportWizard.test.tsx::unknown mapping: preview then mapping step` | Wizard shows mapping UI when `mapping_used=null` |
| **FE** | `ImportWizard.test.tsx::known mapping: preview then straight to reconcile config` | Auto-applied mapping skips mapping step |
| **FE** | `MappingStep.test.tsx` | Column select UI, validation |
| **FE** | `ImportWizard.test.tsx::mapping confirm deletes probe batch before re-upload` | Probe (0 movements) deleted, file re-uploaded with confirmed mapping |

---

## RF-03  Detect and report invalid lines without blocking; keep valid

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `upload_import.rs::upload_import_keeps_invalid_lines` | Bad date / bad amount → `invalid_count=2`, reasons contain "date"/"amount", valid lines still inserted |
| **App** | `csv.rs` | Line-level error collection, partial success |
| **Infra** | `csv_decoder.rs` | Row-level parse with error accumulation |

---

## RF-04  List FF3 asset accounts; select target at import; pre-select at reconcile

**Status:** partial (spec v0.7) · **Last verified:** 2026-09-03
**Pending (v0.7):** account auto-suggestion from the origin-account column (match by IBAN, account number or name; unique match pre-selects, no/multiple match keeps manual selection with a non-blocking warning listing distinct values). The v0.6 deltas (select at import, pre-select at reconcile) are implemented and tested.

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `get_account_transactions.rs` | `Ff3AccountRepository.all_asset()` returns cached accounts |
| **Infra** | `ff3_account_repo.rs` | `Ff3AccountRepository` upsert + query |
| **Infra** | `firefly_client.rs` | `asset_accounts()` calls `GET /api/v1/accounts?type=asset` |
| **Pres** | `firefly_endpoints.rs` | `GET /api/firefly/accounts` returns cached asset accounts |
| **FE** | `ImportWizard.test.tsx` (multiple) | Account selector in step 1, pre-selected in reconcile step |
| **FE** | `ReconcileConfigStep.test.tsx` | Account pre-selected, editable |
| **FE** | `BrowseWidget.test.tsx` | Account picker for period browse (RF-14) |

---

## RF-05  Fetch target account transactions via FF3 API over date window

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `reconcile_batch.rs::fixture_eur` + all tests | `FireflyClient.account_transactions(ff3_id, start, end)` called with window |
| **App** | `get_account_transactions.rs` | `GetAccountTransactions` use case with pagination |
| **Infra** | `firefly_client.rs` | Exhaustive pagination over `GET /api/v1/accounts/{id}/transactions` |
| **Infra** | `reconcile_flow_repo.rs` | End-to-end flow: batch → reconcile → transactions fetched |
| **Pres** | `firefly_endpoints.rs` | `QUERY /api/firefly/accounts/{id}/transactions` with body `{start,end}` |

---

## RF-06  Reconciliation engine with configurable date tolerance

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **Domain** | `matching_engine.rs::exact_match_one_to_one` | Tolerance 0 → exact date match |
| **Domain** | `matching_engine.rs::tolerance_plus_one_two_three` | Tolerance ±1, ±2, ±3 boundaries |
| **Domain** | `matching_engine.rs::chronological_pairing_does_not_cross` | 1-to-1 chronological within window |
| **Domain** | `matching_engine.rs::benchmark_1000_movements_under_5s_nfr04` | NFR-04: 1000 movements < 5 s |
| **App** | `reconcile_batch.rs::reconcile_persists_run_and_updates_statuses` | Full use case with tolerance, window, snapshots |
| **App** | `reconcile_batch.rs::reconcile_currency_mismatch_is_blocked` | Batch currency ≠ account currency → blocked |
| **App** | `reconcile_batch.rs::reconcile_rerun_creates_new_reconciliation` | RF-13 re-reconciliation updates counts |
| **App** | `exact_user_rows.rs` | Real-world row scenarios |

---

## RF-07  Results in 3 groups (matched / missing / extras) with visual gap highlighting

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **Domain** | `matching_engine.rs` (all) | Engine returns `matches`, `missing`, `extra`, `duplicates` |
| **App** | `reconcile_batch.rs::reconcile_persists_run_and_updates_statuses` | Counts: matched/missing/extra/duplicate persisted |
| **App** | `get_results.rs` | `GetResults` use case returns 3 groups + ignored |
| **Infra** | `reconciliation_repo.rs` | `ReconciliationRepository` saves run with snapshot |
| **Pres** | `reconcile_endpoints.rs` | `GET /api/imports/{id}/results` returns grouped DTOs |
| **FE** | `ResultsPage.test.tsx` | Three tabs/sections: Matched, Missing, Extras |
| **FE** | `MissingRow.test.tsx` | Row rendering, checkbox, editable fields |
| **FE** | `ExtraRow.test.tsx` | Duplicate flag display |
| **FE** | `VirtualList.test.tsx` | Large list performance |

---

## RF-08  Batch-create missing; edit description/category/budget/tags/attachment/notes

**Status:** partial (spec v0.7) · **Last verified:** 2026-09-03
**Pending (v0.6):** per-movement inline edit UI (description, category, budget, tags, notes, attachment) and FF3-sourced pickers (categories, budgets, tags) — current ApplyPanel only offers batch-level free-text inputs, no tags picker, no attachment.
**Pending (v0.7):** two-action flow (personalize / create), bulk personalization panel with fill-only semantics, confirmation modal summarizing what will be created, tags pre-filled with the default creation tag (RF-19).

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `apply_missing.rs::apply_creates_missing_with_origin_tag_and_confirms_matched` | Creates in FF3 with origin tag `ffm:filename:batchId`, category, notes |
| **App** | `apply_missing.rs::apply_partial_failure_continues_batch` | Per-movement report, batch continues on failure |
| **App** | `apply_missing.rs::apply_retry_creates_nothing_nfr05` | Idempotency: `creations` table UNIQUE prevents double creation (NFR-05) |
| **App** | `apply_missing.rs::apply_rejects_movement_not_in_missing_state` | Only `Missing` movements accepted |
| **Infra** | `deleted_audit_repo.rs` | Not directly related (deletion audit) |
| **Pres** | `apply_delete_endpoints.rs` | `POST /api/imports/{id}/apply` with selection body |
| **FE** | `ApplyPanel.test.tsx` | Apply flow: selection, counterparty FF3 account picker, batch-level free-text category/budget/notes, mixed-outcome toast recap |
| **FE** | `MissingRow.test.tsx` | Row rendering (description, amount), ignore action, failure/already-created pills — **inline edit NOT implemented/tested (v0.6 gap)** |
| **FE** | `ResultsPage.test.tsx` | "Select all" + apply flow |

---

## RF-09  Each import batch immutable and viewable a posteriori (history)

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `imports_listing.rs` | `ListImports` returns all batches with status |
| **App** | `upload_import.rs` (batch created with `status=Parsed`) | Batch persisted with all metadata |
| **Infra** | `batch_repo.rs` | `BatchRepository` CRUD + `find_by_filename` |
| **Pres** | `import_endpoints.rs` | `GET /api/imports` + `GET /api/imports/{id}` |
| **FE** | `ImportsPage.test.tsx` | History table with status badges, resume/delete actions |

---

## RF-10  Manual ignore / un-ignore; dedicated section in results

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `ignore_movement.rs` | `IgnoreMovement` use case toggles `status=Ignored` |
| **App** | `reconcile_batch.rs::reconcile_persists_run_and_updates_statuses` | Ignored movements excluded from matching, status untouched |
| **Pres** | `reconcile_endpoints.rs` | Ignore endpoint (if exists) |
| **FE** | `ResultsPage.test.tsx` | Ignored section rendered, un-ignore action |
| **FE** | `MissingRow.test.tsx` | Ignore button on missing rows |

---

## RF-11  Created transactions carry origin trace (tag/note: file + batch)

**Status:** partial (spec v0.7) · **Last verified:** 2026-09-03
**Pending (v0.7):** origin tag applied *in addition to* user-selected or default tags, never replaced — the apply use case currently hardcodes `tags: vec![origin_tag]` (`apply_missing.rs:205`); no user-tags input or merge logic exists yet.

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `apply_missing.rs::apply_creates_missing_with_origin_tag_and_confirms_matched` | Tags = `["ffm:stmt.csv:1"]` (origin tag only — merge with user/default tags not covered) |
| **App** | `apply_missing.rs::apply_retry_creates_nothing_nfr05` | Origin tag consistent on retry |

---

## RF-12  Multiple currencies: statement currency from FF3 account; blocking warning on mismatch

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `reconcile_batch.rs::reconcile_currency_mismatch_is_blocked` | Batch `currency` (derived from FF3 account) ≠ account currency → `InvalidState` |
| **App** | `upload_import.rs` | Currency passed from frontend (derived from selected account) |
| **FE** | `ImportWizard.test.tsx` | Account selector shows currency; frontend sends account currency |

> **Amendment v0.4**: Currency no longer manually selected at import; derived from FF3 account.

---

## RF-13  Re-run reconciliation on existing batch

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `reconcile_batch.rs::reconcile_rerun_creates_new_reconciliation` | Second run creates new `Reconciliation` row, updates movement statuses |
| **App** | `get_results.rs` | Can fetch latest or specific run |
| **Pres** | `reconcile_endpoints.rs` | `POST /api/imports/{id}/reconcile` idempotent (creates new run) |
| **FE** | `ReconcileConfigStep.test.tsx` | "Re-run" button on already-reconciled batch |

---

## RF-14  Browse FF3 account transactions by period (min/max date)

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `get_account_transactions.rs` | `GetAccountTransactions` use case with start/end |
| **Infra** | `firefly_client.rs` | Pagination with `start`/`end` query params |
| **Pres** | `firefly_endpoints.rs` | `QUERY /api/firefly/accounts/{id}/transactions` with body |
| **FE** | `BrowseWidget.test.tsx` | Date pickers, account selector, transaction table |

---

## RF-15  Flag duplicates; delete with double confirmation + audit snapshot

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **Domain** | `matching_engine.rs::duplicate_ff3_within_window` | Multiple FF3 within window → 1 match + duplicates flagged |
| **Domain** | `matching_engine.rs::duplicate_of_already_matched_movement` | 3 FF3 for 2 movements → 2 matches + 1 duplicate |
| **App** | `reconcile_batch.rs::reconcile_persists_run_and_updates_statuses` | `is_duplicate=true` on cached FF3 transactions |
| **App** | `delete_firefly_transaction.rs` | `DeleteFireflyTransaction` use case: double confirm, JSON snapshot in `deleted_audit` |
| **Infra** | `deleted_audit_repo.rs` | `DeletedAuditRepository` inserts snapshot |
| **Pres** | `apply_delete_endpoints.rs` | `DELETE /api/firefly/transactions/{id}` with confirmation |
| **FE** | `DeleteDuplicateDialog.test.tsx` | Double confirmation dialog, recap, snapshot preview |
| **FE** | `ResultsPage.test.tsx` / `ExtraRow.test.tsx` | Duplicate badge, delete button |

---

## RF-16  Refuse duplicate filename upload (409 before storage)

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `upload_import.rs::upload_import_rejects_duplicate_filename` | Second upload same filename → `Conflict` error, 0 new batches |
| **App** | `imports_listing.rs` | `find_by_filename` used for check |
| **Infra** | `batch_repo.rs` | `BatchRepository.find_by_filename` (case-insensitive via MySQL collation) |
| **Pres** | `import_endpoints.rs` | `POST /api/imports` returns 409 before insert |
| **FE** | `ImportWizard.test.tsx::duplicate filename is blocked before upload` | Client fetches imports list, blocks upload if filename exists, no POST sent |

---

## RF-17  Delete `parsed` batch after explicit confirmation (file name + count); irreversible; refused for `reconciled`/`applied`

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `delete_import.rs::delete_import_removes_parsed_batch_and_movements` | Batch in `Parsed` status → batch + movements deleted |
| **App** | `delete_import.rs::delete_import_rejects_non_parsed_batch` | `Reconciled`/`Applied` → `InvalidState`, nothing deleted |
| **App** | `delete_import.rs::delete_import_missing_batch_is_not_found` | Non-existent → `NotFound` |
| **Pres** | `import_endpoints.rs` | `DELETE /api/imports/{id}` returns 409 if not `Parsed` |
| **FE** | `ImportsPage.test.tsx` | Delete button only on `Parsed` rows, confirmation modal with filename + movement count |

---

## RF-18  Resume incomplete import: wizard reopens at first pending step (reconcile); free back/forward navigation

**Status:** implemented · **Last verified:** 2026-09-03

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `imports_listing.rs` | Batches with status `Parsed` listed as resumable |
| **FE** | `ImportWizard.test.tsx::resumeBatchId initializes wizard at reconcile-config step` | `resumeBatchId` prop skips to step 2 (reconcile config) |
| **FE** | `ImportWizard.test.tsx` (multiple) | Back/Next buttons work between steps |
| **FE** | `ImportWizard.test.tsx` | State resets on every open (no stale step) |
| **FE** | `WizardShell.test.tsx` | Step navigation logic |

---

## RF-19  App settings: configure FF3 connection (URL + token), verify connectivity, save; replaces env vars

**Status:** pending (spec v0.7) · **Last verified:** 2026-09-03
**Pending (v0.6):** FF3 connection form in settings (URL + token + "Test connection"), `app_settings` storage (write-only token), settings use case + API endpoints, backend reading the connection from DB instead of env vars (`main.rs:37-38` still reads `FF3_URL`/`FF3_TOKEN`).
**Pending (v0.7):** default creation tag (optional, FF3-sourced dropdown), applied by RF-08.

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | (no settings use case exists) | — |
| **Infra** | `firefly_client.rs` | `about()` health probe — env-var config only, not settings-based verification |
| **Pres** | `health.rs` | `GET /health` with `FF3_HEALTH_PROBE=true` calls FF3 `/about` — env-var config only |
| **FE** | `SettingsPage.test.tsx` | Appearance/language sections only — **FF3 connection form NOT implemented** |
| **FE** | — | Token-not-exposed guarantee untestable until the connection form exists |

> **Amendment v0.6**: Settings replace `FF3_URL`/`FF3_TOKEN` env vars. Backend reads from DB at startup.

---

## Cross-cutting Validations

| Concern | Tests |
| --- | --- |
| **Idempotency (NFR-05)** | `apply_missing.rs::apply_retry_creates_nothing_nfr05` (creation UNIQUE), `upload_import.rs::upload_import_rejects_duplicate_filename` |
| **Amounts as cents (NFR-06)** | `matching_engine.rs` (all use `AmountCents::new(cents)`), `value_objects.rs` |
| **No auto create/delete (O4, O5)** | `apply_missing.rs` requires explicit selection; `delete_firefly_transaction.rs` requires double confirm |
| **Security (NFR-02)** | `firefly_client.rs` token from backend env config only; token-not-returned test pending with RF-19 settings |
| **Batch immutability** | `delete_import.rs` rejects non-`Parsed`; `apply_missing.rs` requires `Reconciled` batch |

---

## Running the Validations

```bash
# Backend: all layers
cd backend
cargo test --workspace

# Specific layers
cargo test -p ffm-domain          # Domain (matching engine)
cargo test -p ffm-application     # Use cases
cargo test -p ffm-infrastructure  # Repos, Firefly client
cargo test -p ffm-presentation    # HTTP endpoints

# Frontend
cd frontend
pnpm test                         # Vitest + MSW (no backend needed)

# Full CI gate
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
cd ../frontend && pnpm lint && pnpm test && pnpm build
```

---

## Coverage Summary (as of 2026-09-03)

The per-layer columns reflect **test existence** for the original (pre-amendment) scope; the **Status** column reflects the *amended* spec (see per-RF sections for pending deltas).

| RF | Domain | App | Infra | Pres | FE | Status |
| --- | :---: | :---: | :---: | :---: | :---: | --- |
| 01 | — | ✅ | ✅ | ✅ | ✅ | implemented |
| 02 | — | ✅ | ✅ | ✅ | ✅ | partial (v0.7) |
| 03 | — | ✅ | ✅ | — | — | implemented |
| 04 | — | ✅ | ✅ | ✅ | ✅ | partial (v0.7) |
| 05 | — | ✅ | ✅ | ✅ | ✅ | implemented |
| 06 | ✅ | ✅ | — | — | — | implemented |
| 07 | ✅ | ✅ | ✅ | ✅ | ✅ | implemented |
| 08 | — | ✅ | — | ✅ | ⚠️ | partial (v0.7) |
| 09 | — | ✅ | ✅ | ✅ | ✅ | implemented |
| 10 | — | ✅ | — | — | ✅ | implemented |
| 11 | — | ✅ | — | — | — | partial (v0.7) |
| 12 | — | ✅ | — | — | ✅ | implemented |
| 13 | — | ✅ | — | ✅ | ✅ | implemented |
| 14 | — | ✅ | ✅ | ✅ | ✅ | implemented |
| 15 | ✅ | ✅ | ✅ | ✅ | ✅ | implemented |
| 16 | — | ✅ | ✅ | ✅ | ✅ | implemented |
| 17 | — | ✅ | — | ✅ | ✅ | implemented |
| 18 | — | ✅ | — | — | ✅ | implemented |
| 19 | — | — | ✅* | ✅* | ⚠️ | pending (v0.7) |

**✅ = covered; — = not applicable; ⚠️ = tests exist but do not cover the amended scope**
**\* RF-19 Infra/Pres ✅ = health probe via env-var config only — not the amended settings requirement**

---

*Generated from PRD v0.7 / TRD v0.5 and the current test suite. Status audited 2026-09-03.*