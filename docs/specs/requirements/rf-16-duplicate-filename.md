# RF-16 - Duplicate Filename Refusal

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Priority | P1 |
| Use cases | CU-1 |
| Depends on | RF-01 (upload), RF-09 (existing batches) |
| Related | RF-18 (unambiguous resume), NFR-09 |

## Description

Uploading a file whose **name matches an existing batch** MUST be refused **before any data is stored**, with a clear error message. Uniqueness is enforced at two levels:

- **Client-side pre-check**: before uploading, the wizard checks the filename against the batch list and blocks the upload with an inline error - no request is sent.
- **Server-side check**: at upload time, the system checks the filename against existing batches and refuses the upload if it exists, before any data is stored. Comparison is case-insensitive.

Rationale: duplicate names would make the import history ambiguous and break resume semantics (risk table, PRD section 9).

## Preconditions

1. The user is uploading a file (RF-01).
2. At least one batch exists with a filename (case-insensitively) equal to the uploaded file's name - for the refusal path.

## Steps

1. **Given** a batch named `stmt.csv` exists,
2. **When** the user selects another file named `stmt.csv` in the wizard,
3. **Then** the client blocks the upload with an inline error and sends no request.
4. **Given** an upload that bypasses the client-side check,
5. **Then** the system refuses the upload before storing anything, and no new batch is created.

## Expected result

- The batch list never contains two batches with the same filename.

## Exceptions

- None: refusal is deterministic on both levels.

## Acceptance criteria

- [ ] A second upload with the same filename is rejected and creates no batch
- [ ] The system detects an existing batch by filename (case-insensitively)
- [ ] Nothing is stored on refusal
- [ ] The wizard blocks the upload locally with an inline error, without sending it

## Notes

- The mechanism for case-insensitive comparison is specified in the TRD (section 2).
