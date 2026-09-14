# RF-04 - Asset Account Selection

| Field | Value |
| --- | --- |
| Version | v0.8 (integrates v0.6 + v0.7 + v0.8 amendments) |
| Priority | P0 |
| Use cases | CU-1, CU-3, CU-7 |
| Depends on | RF-19 (FF3 connection) |
| Related | RF-02 (origin-account column), RF-12 (currency derivation), RF-14 (browsing), NFR-09 |

## Description

The system MUST list the Firefly III **asset accounts** so the user can choose the reconciliation target. The target account MUST be selected **during the import** (wizard step 1) and MUST be **pre-selected when running the reconciliation**, where it is displayed as **locked** (read-only): the wizard no longer offers an account selector at reconcile time — to reconcile against a different account, the user deletes the `parsed` batch (RF-17) and re-imports the statement. The account selector MUST display the account currency. Accounts are read through the backend proxy and cached locally.

**Account auto-suggestion**: when the imported file carries an origin-account column (RF-02), the system matches the distinct column values against the Firefly III asset accounts (by IBAN, account number or name):

- **Unique match**: the matched account is pre-selected automatically in the account step; the user can still change it.
- **No match or several distinct values**: selection stays manual and the step displays a **non-blocking warning** listing the distinct origin values found, so mixed-account exports are surfaced.

## Preconditions

1. The Firefly III connection is configured (RF-19) and reachable.

## Steps

1. **Given** the user is on the import wizard's account step,
2. **When** the step loads,
3. **Then** the system lists all Firefly III asset accounts with name and currency.
4. **When** the user selects an account and proceeds,
5. **Then** the batch stores the selected account and its currency becomes the statement currency (RF-12).
6. **Given** the user later opens the reconciliation configuration,
7. **Then** the import-time account is displayed locked (name + read-only hint), with no account selector; the currency-mismatch guard still applies (RF-12).
8. **Given** the imported file carries an origin-account column whose values match exactly one Firefly III asset account,
9. **When** the account step loads,
10. **Then** the matched account is pre-selected automatically (still changeable).

## Expected result

- The account list is complete and cached server-side to avoid repeated FF3 calls.
- The selected account is persisted on the batch.

## Exceptions

- **FF3 API unavailable**: a clear error message is shown with a retry option (NFR-09); the list is not silently empty.
- **No asset accounts exist**: an explicit empty-state message is displayed.
- **Origin values matching no or several asset accounts**: manual selection with a non-blocking warning listing the distinct values found.

## Acceptance criteria

- [ ] The system lists all Firefly III asset accounts, served from the server-side cache
- [ ] The account selector appears only in wizard step 1; the reconcile step displays the import-time account locked (no selector)
- [ ] The account picker for period browsing works
- [ ] A unique origin-account match pre-selects the account automatically (still changeable)
- [ ] Unmatched or multiple distinct origin values keep manual selection, with a non-blocking warning

## Notes

- Integrates v0.6 amendment: account selected at import, pre-selected at reconciliation (CU-1, CU-3).
- Integrates v0.7 amendment: account auto-suggestion from the optional origin-account column (RF-02), matched by IBAN, account number or name.
- v0.8 amendment (2026-09-13): the reconcile step no longer offers an account selector — the import-time account is displayed locked there. Changing the target requires deleting the `parsed` batch (RF-17) and re-importing; the backend currency check (RF-12) remains the API-level guard.
- The selected account is stored on the batch (TRD section 2).
- The account cache is stored server-side (TRD section 2).
