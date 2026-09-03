# RF-03 - Invalid Line Reporting

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Priority | P0 |
| Use cases | CU-1 |
| Depends on | RF-01 (file uploaded), RF-02 (mapping applied or pending) |
| Related | NFR-04 (parse performance) |

## Description

The system MUST detect and report invalid lines (unreadable date, empty or unparseable amount) **without blocking the import**. Valid lines MUST be kept and inserted as movements. Invalid lines MUST be retained with their error attached (raw line + reason) and exposed in the preview, so the user can correct the source file or mapping if desired.

## Preconditions

1. A CSV file has been uploaded (RF-01) and a mapping is applicable (RF-02).

## Steps

1. **Given** a CSV containing a mix of valid and invalid lines,
2. **When** parsing completes,
3. **Then** the valid lines become movements of the batch, and the invalid lines are reported with a per-line reason (e.g. `"date"`, `"amount"`).

## Expected result

- The batch is created with the valid movements.
- The preview exposes the invalid-line count (`invalid_count`) and the reason for each rejected line.
- The import is never blocked by the presence of invalid lines.

## Exceptions

- **All lines invalid**: the batch is created with 0 movements; every line is reported. Import is still not blocked.
- **Headers-only CSV**: 0 movements; treated as a mapping probe (RF-02).

## Acceptance criteria

- [ ] A bad date and a bad amount yield `invalid_count = 2` with reasons containing `"date"`/`"amount"`, and valid lines are still inserted
- [ ] Line-level errors are accumulated with partial success

## Notes

- The full raw line of every movement (valid or invalid) is persisted in `movements.raw_json` for audit and re-parsing (TRD).
