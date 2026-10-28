# RF-15 - Duplicate Flagging and Deletion

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Priority | P0 |
| Use cases | CU-8 |
| Depends on | RF-06 (flagging during matching), RF-07 (extras group) |
| Related | RF-14 (spotting duplicates by browsing), O5, NFR-10 (audit retention) |

## Description

### Flagging

When several Firefly III transactions match the same bank movement (same amount/direction within the date tolerance), the first (chronological) is `matched` and the following ones are flagged **`duplicate`** and surfaced in the extras group with a visible badge (RF-06, RF-07).

### Deletion - unit only, with double confirmation

1. The user clicks **delete** on a transaction flagged `duplicate` (from the reconciliation results or the account browsing view).
2. A confirmation dialog displays the **recap**: amount, date, description, account.
3. The user explicitly confirms → the deletion executes.

No deletion happens without this explicit double confirmation. **No automatic or batch deletion in v1** (objective O5).

### Audit and aftermath

- **Before** the deletion is executed, a snapshot of the transaction's details is saved for audit.
- The system deletes the transaction in Firefly III.
- **After** success, the deleted transaction is removed from the extras list in the current results view.
- Snapshots are retained indefinitely (NFR-10).

## Preconditions

1. A transaction flagged `duplicate` is visible (after a run, or via browsing - RF-14).

## Steps

1. **Given** a duplicate-flagged transaction,
2. **When** the user clicks delete,
3. **Then** the recap dialog appears (amount, date, description, account).
4. **When** the user confirms,
5. **Then** the snapshot is stored, the transaction is deleted in Firefly III, and it disappears from the extras list.

## Expected result

- Exactly one FF3 transaction is deleted per confirmed dialog; the audit snapshot exists for each deletion.

## Exceptions

- **Cancellation**: nothing happens - no snapshot is stored, no deletion request is sent.
- **FF3 deletion error**: the error is surfaced with retry (NFR-09); the snapshot write is rolled back with the operation.
- **Already deleted in FF3**: the error is reported; the UI reflects the actual FF3 state on refresh.

## Acceptance criteria

- [ ] Multiple FF3 matches produce 1 match + duplicates flagged
- [ ] 3 FF3 for 2 movements produce 2 matches + 1 duplicate
- [ ] The duplicate flag is persisted on fetched transactions
- [ ] An audit snapshot is stored before deletion
- [ ] A confirmed deletion removes exactly one FF3 transaction
- [ ] The dialog shows the recap and requires confirmation
- [ ] Duplicate badge + delete button render in results

## Notes

- The pre-deletion audit snapshot is stored server-side (TRD section 2).
- TRD v0.9 (2026-10-27): an FF3 transaction matched split by split cannot be deleted from the app (409) - Firefly III deletes the whole group, including its matched splits.
