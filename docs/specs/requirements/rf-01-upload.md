# RF-01 - CSV Upload

| Field | Value |
| --- | --- |
| Version | v0.1 (2026-09-12) — integrates explicit-Next amendment |
| Priority | P0 |
| Use cases | CU-1 |
| Depends on | RF-19 (FF3 connection configured) |
| Related | RF-02 (mapping), RF-16 (duplicate filename), NFR-09 (error handling) |

## Description

The system MUST allow uploading a CSV file (bank statement) with a maximum size of **10 MB**. Files in any other format (in particular XLSX) MUST be converted to CSV by the user before import; the system MUST NOT accept them. Upload is available via file picker and via drag-and-drop; both entry points MUST behave identically.

## Preconditions

1. The app is running and the Firefly III connection is configured (RF-19).
2. The user is on step 1 of the import wizard (freshly opened, no stale state - RF-18).

## Steps

1. **Given** the user is on the upload step of the import wizard,
2. **When** the user drops or picks a CSV file of ≤ 10 MB,
3. **Then** the file is staged locally on the upload step (file name and size shown); **no upload request is sent** and the wizard stays on step 1,
4. **When** the user confirms with **Next** (enabled only once a file is staged),
5. **Then** the system uploads the file, parses it (RF-02, RF-03), creates a batch in `parsed` status, and displays a preview of the movements.

**Explicit-Next amendment (v0.1, 2026-09-12)**: selecting or dropping a file no longer uploads it. The upload+parse only happens on explicit user confirmation (Next). Re-entering the upload step from preview keeps the staged file; pressing Next again **always re-parses**: the previously created probe batch (if any) is deleted first (RF-17), which frees the file name for RF-16. Discarding the staged file with **Remove** — or replacing it via **Change file** — after a parse deletes the previous probe batch automatically, so no orphan batches remain; if that deletion fails, the error is shown and the staged file and its probe are kept for retry.

## Expected result

- The file is accepted and a batch is created with the parsed movements (or zero movements when manual mapping is required, RF-02).
- The preview shows valid movements and reported invalid lines (RF-03).
- The file name does not collide with an existing batch (RF-16).

## Exceptions

- **File > 10 MB**: the client MUST show an inline error, Next stays disabled, and MUST NOT send the upload request.
- **Non-CSV file**: the upload is refused.
- **Server-side parse error**: an error banner is displayed, the dropzone is reset, and no batch is persisted.
- **Duplicate file name**: the upload is refused before any data is stored (RF-16).

## Acceptance criteria

- [ ] A valid CSV produces a batch and its movements
- [ ] An oversize file shows an inline error and no upload is attempted
- [ ] Drag-and-drop has parity with the picker
- [ ] A parse error shows a banner and resets the dropzone
- [ ] The SHA-256 file hash is computed on upload
- [ ] Selecting or dropping a file sends no request and keeps the wizard on step 1 (explicit-Next amendment)
- [ ] Next is disabled until a file is staged; after going back from preview, Next re-parses (previous probe deleted first)

## Notes

- v0.1 amendment (2026-09-12): explicit-Next confirmation — file selection only stages the file; the upload/parse runs on Next. Re-parse deletes the previous probe batch first, so RF-16 is never violated and stale parses (e.g. after an account/currency change) cannot be used. Remove and Change file clean up the previous probe batch (no orphans).

- XLSX support is intentionally absent in v1 (user converts to CSV); native XLSX/OFX support is planned for V2. No XLSX tests exist by design.
- The uploaded file is fingerprinted (SHA-256) on upload for dedup/traceability (TRD section 2).
