# RF-11 - Origin Trace on Created Transactions

| Field | Value |
| --- | --- |
| Version | v0.8 (amendment: configurable, note instead of tag) |
| Priority | P1 |
| Use cases | CU-5 |
| Depends on | RF-08 (creation flow), RF-19 (connection settings) |
| Related | NFR-05 (provenance survives retries), RF-09 (traceability) |

## Description

Every transaction created by the app MAY carry an **origin trace** - a **note** identifying the source file name and the import batch. The trace makes every app-created transaction identifiable directly inside Firefly III (search by note), enabling audit and cleanup.

The origin trace note is **configurable** via app settings (RF-19): users can enable/disable it. When enabled, it is added **below** the user's note (if any) with line breaks, and is separate from tags (RF-08, RF-19).

## Preconditions

1. A creation is about to run (RF-08).
2. App settings are loaded (RF-19) to check if origin trace is enabled.

## Steps

1. **Given** selected missing movements from batch B imported from file F,
2. **And** origin trace is enabled in app settings (RF-19),
3. **When** the creation succeeds,
4. **Then** each created FF3 transaction carries the origin note identifying F and B.

If origin trace is disabled, no origin note is added.

The origin note is formatted as `ffm:<filename>:<batchId>` (e.g. `ffm:stmt-september.csv:42`) and is appended **below** any user-provided note with line breaks (`\n\n`).

## Expected result

- If enabled: All app-created transactions are searchable in FF3 by their origin note.
- The trace is identical across retries (the movement is only created once, NFR-05).
- The origin note appears below any user note with clear line breaks.
- If disabled: No origin note is added, only user notes appear.

## Exceptions

- None: When enabled, the trace is unconditional on creation.

## Acceptance criteria

- [ ] Created transactions carry an origin note identifying the source file and batch (when enabled)
- [ ] The origin note stays consistent on retry
- [ ] The origin note is added below any user-provided note with line breaks (when enabled)
- [ ] The origin note is separate from tags; tags are not affected by the trace
- [ ] When disabled, no origin note is added

## Notes

- Note format: `ffm:<filename>:<batchId>` (e.g. `ffm:stmt-september.csv:42`) - specified in the TRD (section 3).
- The origin note is always added when enabled, even if the user provides no note.
- If the user provides a note, the origin note is appended below it with two line breaks (`\n\n`).
- Configuration: A setting `enable_origin_trace` (boolean, default: true) in app settings (RF-19).
- **Amendment v0.8**: Changed from tag to note, added configurability (enable/disable).
