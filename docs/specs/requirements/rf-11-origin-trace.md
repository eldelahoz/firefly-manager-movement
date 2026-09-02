# RF-11 - Origin Trace on Created Transactions

| Field | Value |
| --- | --- |
| Priority | P1 |
| Use cases | CU-5 |
| Depends on | RF-08 (creation flow) |
| Related | NFR-05 (provenance survives retries), RF-09 (traceability) |

## Description

Every transaction created by the app MUST carry an **origin trace** - a tag identifying the source file name and the import batch - and MAY be complemented by a note. The trace makes every app-created transaction identifiable directly inside Firefly III (search by tag), enabling audit and cleanup.

The origin trace tag is applied **in addition to** any user-selected or default tags (RF-08, RF-19) and is never replaced by them.

## Preconditions

1. A creation is about to run (RF-08).

## Steps

1. **Given** selected missing movements from batch B imported from file F,
2. **When** the creation succeeds,
3. **Then** each created FF3 transaction carries the origin tag identifying F and B (+ optional notes).

## Expected result

- All app-created transactions are searchable in FF3 by their origin tag.
- The trace is identical across retries (the movement is only created once, NFR-05).

## Exceptions

- None: the trace is unconditional on creation.

## Acceptance criteria

- [ ] Created transactions carry an origin tag identifying the source file and batch
- [ ] The origin tag stays consistent on retry
- [ ] The origin trace tag coexists with user-selected or default tags; it is never replaced by them

## Notes

- Tag format: `ffm:<filename>:<batchId>` (e.g. `ffm:stmt-september.csv:42`) - specified in the TRD (section 3).
