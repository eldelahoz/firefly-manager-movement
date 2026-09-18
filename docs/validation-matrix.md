# RF Validation Matrix — firefly-manager-movement

Maps each **Functional Requirement (RF-01 .. RF-19)** to the tests that verify it.

> **Docs @** PRD `4d18eeb` · TRD `22dad7d` · audited 2026-09-09 — commits of the spec sources this matrix was last checked against (see [Traceability](#traceability)).

> **Legend**
> - **Domain** → `backend/domain/tests/*.rs` (pure logic, no DB)
> - **App** → `backend/application/tests/*.rs` (use cases with fake repos)
> - **Infra** → `backend/infrastructure/tests/*.rs` (Diesel repos, Firefly client)
> - **Pres** → `backend/presentation/tests/*.rs` (axum HTTP handlers)
> - **FE** → `frontend/tests/specs/**/*.spec.ts` (Playwright, `page.route` mocks — Vitest/MSW purged 2026-09-09; see `docs/superpowers/plans/2026-09-09-purge-vitest-only-playwright.md` for gaps)

> **Status** reflects the real implementation state against the *current* spec (including amendments), not just test existence:
> - `implemented` — matches the current spec
> - `partial (spec v0.X)` — base works; the listed amendment deltas are pending
> - `pending (spec v0.X)` — the amended functionality is not implemented
> - `not tested` — no dedicated tests yet (NFR table only)
>
> Spec versions are declared in the `Version` field of each requirement doc (`docs/specs/requirements/`): `v0.X (integrates ...)` for amended requirements, `Original (2026-08-14) — unamended` otherwise.

---

## RF-01  CSV upload (max 10 MB); XLSX → CSV by user

**Status:** implemented · **Last verified:** 2026-09-12 · **Spec @** `220ea85`

> **v0.1 amendment delta (2026-09-12, explicit-Next)**: file selection/drop only stages the file (no request, wizard stays on step 1); upload+parse runs only on Next; Next disabled until a file is staged; Back from preview keeps the staged file and Next re-parses (previous probe batch deleted first); Remove and Change file delete the previous probe batch (no orphan batches). Backend upload/parse semantics unchanged.

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `upload_import.rs::upload_import_parses_valid_csv` | Valid CSV → batch + movements created |
| **App** | `upload_import.rs::upload_import_keeps_invalid_lines` | Invalid lines reported, valid kept (also RF-03) |
| **App** | `upload_import.rs::upload_import_computes_file_hash` | SHA-256 hash computed for dedup |
| **App** | `upload_import.rs::upload_import_rejects_duplicate_filename` | 409 Conflict before any data stored (also RF-16) |
| **App** | `csv.rs` (multiple) | CSV parsing, delimiter detection, encoding |
| **Infra** | `csv_decoder.rs` | Low-level CSV decode, delimiter sniffing |
| **FE** | `import-wizard.spec.ts` W5 | Oversize guard: inline error, no upload request attempted |
| **FE** | `import-wizard.spec.ts` W3, W3b | Drag-and-drop parity with the picker + dragover animation |
| **FE** | `import-wizard.spec.ts` W4 | Duplicate filename refused at staging (case-insensitive), Next stays disabled |
| **FE** | `import-wizard.spec.ts` W19 | Explicit-Next (v0.1): staging sends no request and stays on step 1; Next disabled without file; confirm parses exactly once |
| **FE** | `import-wizard.spec.ts` W20 | Remove after a parse deletes the probe batch (no orphans); Remove without parse sends no request |

**No XLSX tests** — by design (RF-01: user converts to CSV).

---

## RF-02  CSV parsing with manual mapping; memorize by header signature

**Status:** implemented (spec v0.8) · **Last verified:** 2026-09-18 · **Spec @** `189309b`

**v0.8 delta (explicit date format) — implemented + verified**: precedence explicit > memorized-by-signature > settings `default_date_format` > auto-try; exclusive parsing with invalid lines naming the format (RF-03); format persisted inside `mapping_json` and reapplied by signature (mapping step skipped); settings default round-trip; mapping-step preview + ambiguity warning. Pre-v0.8 scope unchanged (rows below, all green).

| Layer | Test | What it validates |
| --- | --- | --- |
| **Domain** | `value_objects.rs::date_format` (4 tests) | `DateFormat`: canonical `as_str`, chrono format, `is_ambiguous` (both dd/mm & mm/dd readings valid), `FromStr` rejects unknown |
| **App** | `upload_import.rs::upload_import_no_mapping_returns_headers_only` | Unknown headers → batch created, 0 movements, headers returned |
| **App** | `upload_import.rs::upload_import_uses_provided_mapping` | Explicit `ColumnMapping` used |
| **App** | `upload_import.rs::upload_import_uses_saved_mapping_by_signature` | Header signature lookup → auto-apply saved mapping |
| **App** | `upload_import.rs::upload_import_extracts_origin_account_values` | Origin-account column values parsed into movements |
| **App** | `upload_import.rs::upload_import_empty_origin_account_values_when_no_column` | No origin column → `origin_account` is `None` |
| **App** | `upload_import.rs::upload_import_auto_detects_spanish_headers_valor` | Known header layouts auto-detected |
| **App** | `upload_import.rs::upload_import_persists_headers_and_delimiter_on_batch` | Headers + delimiter persisted on the batch (resume mapping source) |
| **App** | `csv.rs::explicit date format` (6 tests) | Explicit format parses exclusively; non-matching value → invalid line naming the format; `05/01/2026` → 2026-05-01 (`mm/dd`) / 2026-01-05 (`dd/mm`); no-format falls back to legacy auto-try |
| **App** | `upload_import.rs::date_format precedence` (4 tests) | explicit > memorized-by-signature > settings default > auto-try |
| **App** | `app_settings.rs::default date format` (4 tests) | Save / clear / validate `default_date_format` |
| **Infra** | `csv_mapping_repo.rs` | `CsvMappingRepository` CRUD + `by_signature` (JSON carries `date_format`) |
| **Infra** | `app_settings_repo.rs` | `default_date_format` roundtrip + reversible migration replay |
| **Infra** | `batch_repo.rs::batch_repo_roundtrip` | Headers/delimiter JSON roundtrip in MySQL |
| **Pres** | `import_endpoints.rs::get_import_returns_batch_and_movements` | `GET /{id}` returns persisted `headers` + `detected_delimiter` |
| **Pres** | `import_endpoints.rs::upload_with_date_format_parses_exclusively_and_reports_it` | Multipart `date_format` drives exclusive parsing |
| **Pres** | `import_endpoints.rs::upload_with_unknown_date_format_returns_400` | Unknown wire value → 400 |
| **Pres** | `import_endpoints.rs::put_mapping_with_date_format_persists_and_unknown_is_400` | `PUT /mapping` persists format by signature; unknown → 400 |
| **FE** | `import-wizard.spec.ts` W2, W16 | Manual mapping flow: probe → map → re-upload with columns |
| **FE** | `import-wizard.spec.ts` W22 | Resume: mapping step lists the persisted CSV columns |
| **FE** | `import-wizard-date-format.spec.ts` DF1–DF5 | Preview per format + `mm/dd` vs `dd/mm` readings, ambiguity warning, PUT body + multipart field, signature reapply skips mapping, settings pre-fill |

---

## RF-03  Detect and report invalid lines without blocking; keep valid

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `95378a6`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `upload_import.rs::upload_import_keeps_invalid_lines` | Bad date / bad amount → `invalid_count=2`, reasons contain "date"/"amount", valid lines still inserted |
| **App** | `csv.rs` | Line-level error collection, partial success |
| **Infra** | `csv_decoder.rs` | Row-level parse with error accumulation |

---

## RF-04  List FF3 asset accounts; select target at import; locked at reconcile

**Status:** implemented · **Last verified:** 2026-09-13 · **Spec @** `4d18eeb`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `get_account_transactions.rs` | `Ff3AccountRepository.all_asset()` returns cached accounts |
| **App** | `upload_import.rs::upload_import_stores_target_account` | Upload persists the selected account on the batch (FF3 id → local id, RF-04 v0.8) |
| **Infra** | `ff3_account_repo.rs` | `Ff3AccountRepository` upsert + query (includes IBAN/number) |
| **Infra** | `firefly_client.rs` | `asset_accounts()` calls `GET /api/v1/accounts?type=asset` |
| **Pres** | `firefly_endpoints.rs` | `GET /api/firefly/accounts` returns cached asset accounts |
| **FE** | `import-wizard.spec.ts` W7 | Account selector at wizard step 1 (upload) with currency |
| **FE** | `import-wizard.spec.ts` W23 | Fresh reconcile: import-time account shown locked (name + hint, no account combobox) |
| **FE** | `import-wizard.spec.ts` W12 | Resume: account locked hint displayed |
| **FE** | `dashboard-browse.spec.ts` ✅ (BrowseWidget) | Account browse widget |

---

## RF-05  Fetch target account transactions via FF3 API over date window

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `95378a6`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `reconcile_batch.rs::fixture_eur` + all tests | `FireflyClient.account_transactions(ff3_id, start, end)` called with window |
| **App** | `get_account_transactions.rs` | `GetAccountTransactions` use case with pagination |
| **Infra** | `firefly_client.rs` | Exhaustive pagination over `GET /api/v1/accounts/{id}/transactions` |
| **Infra** | `reconcile_flow_repo.rs` | End-to-end flow: batch → reconcile → transactions fetched |
| **Pres** | `firefly_endpoints.rs` | `QUERY /api/firefly/accounts/{id}/transactions` with body `{start,end}` |

---

## RF-06  Reconciliation engine with configurable date tolerance

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `95378a6`

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

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `d734652`

| Layer | Test | What it validates |
| --- | --- | --- |
| **Domain** | `matching_engine.rs` (all) | Engine returns `matches`, `missing`, `extra`, `duplicates` |
| **App** | `reconcile_batch.rs::reconcile_persists_run_and_updates_statuses` | Counts: matched/missing/extra/duplicate persisted |
| **App** | `get_results.rs` | `GetResults` use case returns 3 groups + ignored |
| **Infra** | `reconciliation_repo.rs` | `ReconciliationRepository` saves run with snapshot |
| **Pres** | `reconcile_endpoints.rs` | `GET /api/imports/{id}/results` returns grouped DTOs |
| **FE** | `pending (Playwright gap)` → `tests/specs/results-groups.spec.ts` + `tests/specs/virtual-list.spec.ts` (gaps `results-matched-missing-extra`, `virtual-list`) | 3 groups, headers, VirtualList — pending |

---

## RF-08  Batch-create missing; edit description/category/budget/tags/attachment/notes

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `ae8790d`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `apply_missing.rs::applies_per_movement_customization` | Sign-based type: negative → `withdrawal` (source = batch account), positive → `deposit` (source = counterparty); per-movement description, category, budget, tags, notes override |
| **App** | `apply_missing.rs::applies_without_counterparty_leaves_side_unset` | Creation works without counterparty (optional field, v0.8) |
| **Infra** | `firefly_client.rs::create_transaction_sends_absolute_amount` | FF3 amount always positive; direction via type + accounts |
| **Infra** | `firefly_client.rs::create_transaction_omits_absent_account_side` | FF3 payload omits absent account side (v0.8) |
| **App** | `apply_missing.rs::apply_creates_with_origin_tag_and_confirms_matched` | Creates in FF3 with origin tag, category, notes |
| **App** | `apply_missing.rs::origin_tag_appended_in_addition_to_user_tags` | Origin tag merged with user-selected tags |
| **App** | `apply_missing.rs::origin_tag_appended_when_user_tags_empty` | Origin tag when no user tags; default_creation_tag from settings applied when configured |
| **App** | `apply_missing.rs::origin_tag_not_duplicated_when_user_includes_it` | No duplicate origin tag |
| **App** | `apply_missing.rs::zero_amount_movement_fails_without_ff3_call` | Zero-amount validation before FF3 call |
| **App** | `apply_missing.rs::attachment_uploaded_after_creation` | Attachment uploaded after transaction creation |
| **App** | `apply_missing.rs::attachment_failure_does_not_fail_creation` | Attachment failure doesn't block creation |
| **App** | `apply_missing.rs::apply_partial_failure_continues_batch` | Per-movement report, batch continues on failure |
| **App** | `apply_missing.rs::apply_retry_creates_nothing_nfr05` | Idempotency: `creations` table UNIQUE prevents double creation (NFR-05) |
| **App** | `apply_missing.rs::apply_rejects_movement_not_in_missing_state` | Only `Missing` movements accepted |
| **Pres** | `apply_delete_endpoints.rs::apply_endpoint_creates_and_reports` | `POST /api/imports/{id}/apply` with selection body |
| **Pres** | `apply_delete_endpoints.rs::apply_endpoint_creates_without_counterparty` | Apply endpoint creates without counterparty (v0.8) |
| **Pres** | `apply_delete_endpoints.rs::apply_partial_failure_continues_and_reports_reason` | Partial failure reporting |
| **Pres** | `apply_delete_endpoints.rs::apply_retry_endpoint_is_idempotent` | Retry returns same results |
| **FE** | `pending (Playwright gap)` → `tests/specs/results-apply.spec.ts` (gap `results-apply-personalize`) | Batch-create missing, edit fields, origin tag — pending |

---

## RF-09  Each import batch immutable and viewable a posteriori (history)

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `95378a6`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `imports_listing.rs` | `ListImports` returns all batches with status |
| **App** | `upload_import.rs` (batch created with `status=Parsed`) | Batch persisted with all metadata |
| **Infra** | `batch_repo.rs` | `BatchRepository` CRUD + `find_by_filename` |
| **Pres** | `import_endpoints.rs` | `GET /api/imports` + `GET /api/imports/{id}` |
| **FE** | `pending (Playwright gap)` → `tests/specs/imports-resume-delete.spec.ts` (gap `imports-resume-delete`) | History table, status badges — pending |

---

## RF-10  Manual ignore / un-ignore; dedicated table below the results

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `d734652`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `ignore_movement.rs` | `IgnoreMovement` use case sets `status=Ignored` |
| **App** | `ignore_movement.rs::un_ignore_restores_missing_when_not_in_latest_run_matches` | Un-ignore restores `Missing` when latest run didn't match |
| **App** | `ignore_movement.rs::un_ignore_restores_matched_when_in_latest_run_matches` | Un-ignore restores `Matched` when latest run matched |
| **App** | `ignore_movement.rs::un_ignore_without_any_run_restores_imported` | Un-ignore restores `Imported` when no run exists |
| **App** | `reconcile_batch.rs::reconcile_persists_run_and_updates_statuses` | Ignored movements excluded from matching, status untouched |
| **App** | `get_results.rs::results_include_ignored_movements` | Results include the ignored movements |
| **Pres** | `reconcile_endpoints.rs::results_include_ignored_movements` | Endpoint returns ignored movements in results |
| **Pres** | `reconcile_endpoints.rs` | Ignore endpoint sets the ignored state |
| **Pres** | `reconcile_endpoints.rs::un_ignore_endpoint_restores_latest_run_status` | Un-ignore endpoint restores status per latest run |
| **FE** | `pending (Playwright gap)` → `tests/specs/results-ignore.spec.ts` (gap `results-ignore-unignore`) | Ignore/un-ignore, dedicated section — pending |

---

## RF-11  Created transactions carry origin trace (tag/note: file + batch)

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `2d7604a`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `apply_missing.rs::apply_creates_with_origin_tag_and_confirms_matched` | Tags = `["ffm:stmt.csv:1"]` (origin tag) |
| **App** | `apply_missing.rs::origin_tag_appended_in_addition_to_user_tags` | Origin tag merged with user tags, not replaced |
| **App** | `apply_missing.rs::origin_tag_appended_when_user_tags_empty` | Origin tag applied even with empty user tags |
| **App** | `apply_missing.rs::origin_tag_not_duplicated_when_user_includes_it` | No duplicate when user includes origin tag |
| **App** | `apply_missing.rs::apply_retry_creates_nothing_nfr05` | Origin tag consistent on retry |
| **FE** | `pending (Playwright gap)` → `tests/specs/results-apply.spec.ts` (gap `results-apply-personalize`) | Origin trace tag format — pending |

---

## RF-12  Multiple currencies: statement currency from FF3 account; blocking warning on mismatch

**Status:** implemented · **Last verified:** 2026-09-13 · **Spec @** `4d18eeb`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `reconcile_batch.rs::reconcile_currency_mismatch_is_blocked` | Batch `currency` (derived from FF3 account) ≠ account currency → `InvalidState` |
| **App** | `upload_import.rs` | Currency passed from frontend (derived from selected account) |
| **FE** | `import-wizard.spec.ts` W7 | Mismatch (EUR account vs USD batch) → blocking warning + reconcile disabled |

> **Amendment v0.4**: Currency no longer manually selected at import; derived from FF3 account.
> **Amendment v0.5** (2026-09-13): account locked at reconcile (RF-04 v0.8); API mismatch rule unchanged.

---

## RF-13  Re-run reconciliation on existing batch

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `95378a6`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `reconcile_batch.rs::reconcile_rerun_creates_new_reconciliation` | Second run creates new `Reconciliation` row, updates movement statuses |
| **App** | `get_results.rs` | Can fetch latest or specific run |
| **Pres** | `reconcile_endpoints.rs` | `POST /api/imports/{id}/reconcile` idempotent (creates new run) |
| **FE** | `pending (Playwright gap)` → `tests/specs/import-wizard.spec.ts` (gap `imports-upload-mapping-reconcile`) | Re-run reconciliation — pending |

---

## RF-14  Browse FF3 account transactions by period (min/max date)

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `95378a6`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `get_account_transactions.rs` | `GetAccountTransactions` use case with start/end |
| **Infra** | `firefly_client.rs` | Pagination with `start`/`end` query params |
| **Pres** | `firefly_endpoints.rs` | `QUERY /api/firefly/accounts/{id}/transactions` with body |
| **FE** | `pending (Playwright gap)` → `tests/specs/dashboard-browse.spec.ts` ✅ (gap `dashboard-browse`) | Browse by period — ✅ covered (6 tests) |

---

## RF-15  Flag duplicates; delete with double confirmation + audit snapshot

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `95378a6`

| Layer | Test | What it validates |
| --- | --- | --- |
| **Domain** | `matching_engine.rs::duplicate_ff3_within_window` | Multiple FF3 within window → 1 match + duplicates flagged |
| **Domain** | `matching_engine.rs::duplicate_of_already_matched_movement` | 3 FF3 for 2 movements → 2 matches + 1 duplicate |
| **App** | `reconcile_batch.rs::reconcile_persists_run_and_updates_statuses` | `is_duplicate=true` on cached FF3 transactions |
| **App** | `delete_firefly_transaction.rs` | `DeleteFireflyTransaction` use case: double confirm, JSON snapshot in `deleted_audit` |
| **App** | `get_results.rs::results_exclude_deleted_ff3_transactions_from_extra` | Deleted FF3 tx removed from extras list after deletion (spec: "removed from the extras list") |
| **Infra** | `deleted_audit_repo.rs::insert_returns_increasing_ids` | `DeletedAuditRepository` inserts snapshot + `deleted_ff3_ids` lists deleted ids |
| **Infra** | `firefly_client.rs::delete_transaction_not_found_maps_to_not_found` | Already-deleted FF3 tx → `NotFound` (API 404, spec exception reported) |
| **Pres** | `apply_delete_endpoints.rs` | `DELETE /api/firefly/transactions/{id}` with confirmation |
| **FE** | `pending (Playwright gap)` → `tests/specs/browse-delete-duplicate.spec.ts` (gap `browse-delete-duplicate`) | Duplicate delete double confirm + audit — pending |

---

## RF-16  Refuse duplicate filename upload (409 before storage)

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `95378a6`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `upload_import.rs::upload_import_rejects_duplicate_filename` | Second upload same filename → `Conflict` error, 0 new batches |
| **App** | `imports_listing.rs` | `find_by_filename` used for check |
| **Infra** | `batch_repo.rs` | `BatchRepository.find_by_filename` (case-insensitive via MySQL collation) |
| **Pres** | `import_endpoints.rs` | `POST /api/imports` returns 409 before insert |
| **FE** | `pending (Playwright gap)` → `tests/specs/import-wizard.spec.ts` (gap `imports-upload-mapping-reconcile`) | Duplicate filename 409 guard — pending |

---

## RF-17  Delete `parsed` batch after explicit confirmation (file name + count); irreversible; refused for `reconciled`/`applied`

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `95378a6`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `delete_import.rs::delete_import_removes_parsed_batch_and_movements` | Batch in `Parsed` status → batch + movements deleted |
| **App** | `delete_import.rs::delete_import_rejects_non_parsed_batch` | `Reconciled`/`Applied` → `InvalidState`, nothing deleted |
| **App** | `delete_import.rs::delete_import_missing_batch_is_not_found` | Non-existent → `NotFound` |
| **Pres** | `import_endpoints.rs` | `DELETE /api/imports/{id}` returns 409 if not `Parsed` |
| **FE** | `pending (Playwright gap)` → `tests/specs/imports-resume-delete.spec.ts` (gap `imports-resume-delete`) | Delete parsed batch confirm — pending |

---

## RF-18  Resume incomplete import: wizard reopens at first pending step (reconcile); free back/forward navigation

**Status:** implemented · **Last verified:** 2026-09-13 · **Spec @** `95378a6`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `imports_listing.rs` | Batches with status `Parsed` listed as resumable |
| **Pres** | `import_endpoints.rs::get_import_returns_batch_and_movements` | Detail feeds the resume probe (batch + movements + persisted headers) |
| **FE** | `import-wizard.spec.ts` W12 | Resume opens at reconcile-config with the import account locked |
| **FE** | `import-wizard.spec.ts` W13 | Probe batches (0 movements) not offered for resume |
| **FE** | `import-wizard.spec.ts` W14 | Wizard state does not persist across reload (fresh resume works) |
| **FE** | `import-wizard.spec.ts` W2-resume | Back → preview → Next → mapping reachable on resume |
| **FE** | `import-wizard.spec.ts` W22 | Mapping step lists the persisted CSV columns on resume |

---

## RF-19  App settings: configure FF3 connection (URL + token), verify connectivity, save; replaces env vars

**Status:** implemented · **Last verified:** 2026-09-09 · **Spec @** `2d7604a`

| Layer | Test | What it validates |
| --- | --- | --- |
| **App** | `app_settings.rs::get_returns_empty_view_when_no_row` | Empty state before first save (no env fallback configured) |
| **App** | `app_settings.rs::get_reflects_env_fallback_when_no_row` | GET with no row returns effective env URL + token presence (TRD §4 fallback) |
| **App** | `app_settings.rs::get_masks_token_but_reports_presence` | Token masked in GET response |
| **App** | `app_settings.rs::save_persists_reconfigures_and_keeps_token_when_omitted` | Save persists, reconfigure FF3 client, keep token when blank |
| **App** | `app_settings.rs::save_replaces_token_when_provided_and_clears_tag` | Token replacement, tag cleared |
| **App** | `app_settings.rs::save_bootstraps_token_from_env_when_no_row` | First save without token input persists env token (bootstrap) |
| **App** | `app_settings.rs::save_rejects_blank_url` | Blank URL → `Validation` error |
| **App** | `app_settings.rs::save_rejects_non_http_url` | Non-HTTP URL → `Validation` error |
| **App** | `app_settings.rs::save_without_any_token_is_rejected` | No token anywhere → `Validation` error |
| **App** | `app_settings.rs::verify_probes_via_factory` | Verify connectivity via factory |
| **App** | `app_settings.rs::verify_rejects_invalid_url` | Invalid URL → `Validation` error |
| **App** | `app_settings.rs::verify_rejects_blank_token` | Blank token → `Validation` error |
| **Infra** | `app_settings_repo.rs` | Repository CRUD + reversible migration + ~1 KB FF3 token persistence (migration 0008) |
| **Pres** | `settings_endpoints.rs::get_settings_returns_url_and_has_token` | GET returns URL + token presence |
| **Pres** | `settings_endpoints.rs::get_settings_reflects_env_fallback_when_no_row` | GET with no row returns env URL + `has_token`, never the token value |
| **Pres** | `settings_endpoints.rs::put_settings_saves_and_reconfigures` | PUT saves and reconfigures |
| **Pres** | `settings_endpoints.rs::put_settings_keeps_existing_token_when_omitted` | Token kept when not provided |
| **Pres** | `settings_endpoints.rs::put_settings_blank_url_returns_400` | Blank URL → 400 |
| **Pres** | `settings_endpoints.rs::verify_ok_returns_version` | Verify → FF3 version |
| **Pres** | `settings_endpoints.rs::verify_bad_url_returns_400` | Invalid URL → 400 |
| **Pres** | `settings_endpoints.rs::verify_unreachable_returns_502` | Unreachable → 502 |
| **Pres** | `settings_endpoints.rs::get_settings_empty_when_no_row` | Empty state (no row, no env token) |
| **FE** | `pending (Playwright gap)` → `tests/specs/settings-*.spec.ts` (gaps `settings-appearance`, `settings-language`, `settings-connection-verify`, `settings-creation-defaults`) | FF3 connection, verify, creation defaults — pending |
| **App** | `apply_missing.rs::origin_tag_appended_when_user_tags_empty` | `default_creation_tag` from settings pre-fills tags when user tags empty |
| **App** | `apply_missing.rs::apply_creates_with_origin_tag_and_confirms_matched` | Origin tag applied even with default tag configured |

---

## Cross-cutting Validations

| Concern | Tests |
| --- | --- |
| **Idempotency (NFR-05)** | `apply_missing.rs::apply_retry_creates_nothing_nfr05` (creation UNIQUE), `upload_import.rs::upload_import_rejects_duplicate_filename` |
| **Amounts as cents (NFR-06)** | `matching_engine.rs` (all use `AmountCents::new(cents)`), `value_objects.rs` |
| **No auto create/delete (O4, O5)** | `apply_missing.rs` requires explicit selection; `delete_firefly_transaction.rs` requires double confirm |
| **Security (NFR-02)** | `firefly_client.rs` token backend-only (env / app settings); settings GET masks token — `app_settings.rs::get_masks_token_but_reports_presence`, `settings_endpoints.rs::get_settings_returns_url_and_has_token` |
| **Batch immutability** | `delete_import.rs` rejects non-`Parsed`; `apply_missing.rs` requires `Reconciled` batch |

---

## NFR Validation Matrix

Per-requirement tracking for `docs/specs/requirements/nfr-*.md`.

| NFR | Concern | Status | Last verified | Spec @ |
| --- | --- | --- | --- | --- |
| 01 | Clean Architecture layering | not tested | — | `95378a6` |
| 02 | Security: backend-only token, masking | implemented | 2026-09-03 | `95378a6` |
| 03 | Privacy: no PII in logs | not tested | — | `95378a6` |
| 04 | Performance: reconcile 1000 movements < 5 s | implemented | 2026-09-03 | `95378a6` |
| 05 | Idempotency: no double creation / duplicate upload | implemented | 2026-09-03 | `95378a6` |
| 06 | Amount integrity: signed cents, no floats | implemented | 2026-09-03 | `95378a6` |
| 07 | Observability | not tested | — | `95378a6` |
| 08 | Internationalization (en/fr/es) | not tested | — | `95378a6` |
| 09 | Error handling | not tested | — | `95378a6` |
| 10 | Data retention | not tested | — | `95378a6` |
| 11 | Accessibility | not tested | — | `95378a6` |
| 12 | Responsive layout | not tested | — | `95378a6` |
| 13 | Browser support | not tested | — | `95378a6` |

---

## Traceability

Each requirement is pinned to the exact commit of its spec file that was reviewed when setting its **Status**:

- **RF sections** → `Spec @` on the status line (`docs/specs/requirements/rf-XX-*.md` commit)
- **NFR table** → `Spec @` column (`docs/specs/requirements/nfr-XX-*.md` commit)
- **Header** → `Docs @` (PRD / TRD commits)

If a spec file changes after its pin, the matrix entry is **stale** and must be re-verified before it can be trusted.

### Checking freshness

Compare the pin against the file's last commit manually:

```bash
git log -1 --format=%h -- docs/specs/requirements/rf-08-batch-creation.md
```

If the output differs from the pinned `Spec @`, re-verify the requirement against its current spec, update the pin (and `Last verified` if re-verified) in the same session, and commit.

### Commit trailers

Commits that touch requirement specs or their implementation carry:

```
Refs: RF-08, NFR-06      # requirement IDs this commit affects
Spec: v0.8               # only when a spec version bumps
```

Query the history of a requirement: `git log --grep "Refs: RF-08" --oneline`.

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
pnpm e2e                          # Playwright (tests/specs/** via page.route, no backend needed)

# Full CI gate
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
cd ../frontend && pnpm lint && pnpm build && pnpm e2e
```

---

## Coverage Summary (as of 2026-09-09 — purge Vitest, solo Playwright)

All RFs are backend-implemented; FE now solo Playwright. `⏳` = pending Playwright gap (see plan 2026-09-09).

| RF | Domain | App | Infra | Pres | FE | Status |
| --- | :---: | :---: | :---: | :---: | :---: | --- |
| 01 | — | ✅ | ✅ | ✅ | ⏳ | implemented (FE pending `imports-upload-mapping-reconcile`) |
| 02 | ✅ | ✅ | ✅ | ✅ | ✅ | implemented (spec v0.8 — date format DF1–DF5) |
| 03 | — | ✅ | ✅ | — | — | implemented |
| 04 | — | ✅ | ✅ | ✅ | ⏳/✅ | implemented (FE BrowseWidget ✅, resto pending) |
| 05 | — | ✅ | ✅ | ✅ | — | implemented |
| 06 | ✅ | ✅ | — | — | — | implemented |
| 07 | ✅ | ✅ | ✅ | ✅ | ⏳ | implemented (FE pending `results-matched-missing-extra`/`virtual-list`) |
| 08 | — | ✅ | ✅ | ✅ | ⏳ | implemented (FE pending `results-apply-personalize`) |
| 09 | — | ✅ | ✅ | ✅ | ⏳ | implemented (FE pending `imports-resume-delete`) |
| 10 | — | ✅ | ✅ | ✅ | ⏳ | implemented (FE pending `results-ignore-unignore`) |
| 11 | — | ✅ | — | — | ⏳ | implemented (FE pending `results-apply-personalize`) |
| 12 | — | ✅ | — | — | ⏳ | implemented (FE pending `imports-upload-mapping-reconcile`) |
| 13 | — | ✅ | — | ✅ | ⏳ | implemented (FE pending `imports-upload-mapping-reconcile`) |
| 14 | — | ✅ | ✅ | ✅ | ✅ | implemented (FE ✅ `dashboard-browse` 6 tests) |
| 15 | ✅ | ✅ | ✅ | ✅ | ⏳ | implemented (FE pending `browse-delete-duplicate`) |
| 16 | — | ✅ | ✅ | ✅ | ⏳ | implemented (FE pending `imports-upload-mapping-reconcile`) |
| 17 | — | ✅ | — | ✅ | ⏳ | implemented (FE pending `imports-resume-delete`) |
| 18 | — | ✅ | — | — | ⏳ | implemented (FE pending `imports-resume-delete`) |
| 19 | — | ✅ | ✅ | ✅ | ⏳ | implemented (FE pending `settings-*`) |

**✅ = covered; ⏳ = pending Playwright gap; — = not applicable**

---

*Generated from PRD v0.8 / TRD v0.7 and the current test suite. Status audited 2026-09-09 — purge Vitest, solo Playwright.*