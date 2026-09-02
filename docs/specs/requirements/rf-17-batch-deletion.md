# RF-17 - Parsed Batch Deletion

| Field | Value |
| --- | --- |
| Priority | P1 |
| Use cases | CU-10 |
| Depends on | RF-09 (batch lifecycle) |
| Related | RF-02 (probe deletion uses this path), NFR-10 (retention rules) |

## Description

The user MUST be able to delete an import batch in **`parsed` status** after an **explicit confirmation** displaying a recap: **file name + movement count**.

- Deletion removes the batch **and all its movements**; it is **irreversible**.
- Deletion is **refused** for batches in `reconciled` or `applied` status (batch immutability, RF-09) - the refusal is explicit and no data is removed.
- The confirmation dialog is explicit (destructive action styling).
- The mapping-probe cleanup of RF-02 uses this same deletion path automatically (no confirmation needed for probes - they are an implementation detail invisible to the user).

## Preconditions

1. The batch exists and is in `parsed` status.
2. The user opened the delete confirmation dialog.

## Steps

1. **Given** a `parsed` batch in the history,
2. **When** the user clicks delete,
3. **Then** a confirmation dialog shows the file name and the movement count.
4. **When** the user confirms,
5. **Then** the batch and its movements are permanently removed and the history refreshes.

## Expected result

- After deletion, the batch no longer appears in the history and can no longer be accessed.

## Exceptions

- **`reconciled`/`applied` batch**: deletion refused; nothing is deleted.
- **Non-existent batch id**: "not found" error.
- **Cancellation**: nothing happens.

## Acceptance criteria

- [ ] `Parsed` batch + movements are deleted
- [ ] `Reconciled`/`Applied` batches are rejected, nothing deleted
- [ ] A missing batch yields a "not found" error
- [ ] The delete button only appears on `Parsed` rows; the modal shows filename + movement count
