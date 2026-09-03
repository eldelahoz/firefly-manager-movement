# RF-02 - CSV Column Mapping

| Field | Value |
| --- | --- |
| Version | v0.7 (integrates v0.5 + v0.7 amendments) |
| Priority | P0 |
| Use cases | CU-2 |
| Depends on | RF-01 (file uploaded) |
| Related | RF-03 (invalid lines), RF-04 (account auto-suggestion), RF-17 (probe deletion), NFR-09 |

## Description

The system parses the CSV using a **column mapping** (CSV columns → fields: date, amount, description, and optionally origin account) and **memorizes mappings by file signature**.

**Origin account column (optional)**: the mapping may bind a CSV column to the **origin account** - the bank account each row belongs to, as exported by the bank. When present, the parsed values feed the account auto-suggestion at import (RF-04). The column is optional: files without it follow the standard manual account selection.

**File signature definition**: the signature is the SHA-256 hash of the **normalized header row** - each header name is trimmed, lowercased and stripped of whitespace, then the normalized names are joined in column order with `;` as separator. Two files whose normalized headers are equal share the same signature.

**Memorization rules**:

- Exactly one mapping is stored per unique signature. The number of memorized mappings is unbounded.
- If a file with the same headers is imported again with a **different** mapping, the newly confirmed mapping **replaces** the stored one for that signature.
- The memorized mapping is applied **automatically** when the headers match a known signature; otherwise manual mapping is required.

**Manual mapping flow (unknown headers)**:

1. The upload returns a headers-only batch (0 movements) and the header list; the preview displays an explanatory message instead of an empty table.
2. The wizard guides the user to the mapping step; the user maps columns → fields and confirms.
3. On confirmation, the mapping is persisted by signature, the headers-only **probe batch is deleted automatically**, and the file is re-uploaded with the confirmed mapping - no orphan batches remain.

## Preconditions

1. A CSV file has been uploaded (RF-01).
2. Its headers either match or do not match a memorized signature.

## Steps

1. **Given** the uploaded CSV headers match a memorized signature,
2. **When** parsing completes,
3. **Then** the saved mapping is applied automatically and the wizard goes straight to the reconciliation configuration step.
4. **Given** the headers match no memorized signature,
5. **When** parsing completes,
6. **Then** a headers-only probe batch is created, the preview shows an explanatory message (0 valid rows), and the wizard opens the mapping step.
7. **When** the user confirms a mapping,
8. **Then** the mapping is saved by signature, the probe batch is deleted, and the file is re-uploaded and parsed with movements.

## Expected result

- Movements are parsed according to the mapping.
- The mapping is memorized for future files with the same normalized headers.

## Exceptions

- **Unmappable CSV** (not enough distinct columns for date/amount): the mapping step blocks progression until the user provides a valid mapping or cancels.
- **Re-upload failure after probe deletion**: the user restarts the upload; the confirmed mapping is already memorized, so the second attempt skips the mapping step.

## Acceptance criteria

- [ ] Unknown headers produce a headers-only batch with 0 movements and the headers returned
- [ ] An explicitly provided mapping is used
- [ ] A saved mapping is auto-applied by signature
- [ ] The confirmed mapping is persisted by signature
- [ ] Known header layouts (e.g. Spanish `Fecha;Valor` sets) are auto-detected
- [ ] Known mapping skips the mapping step; unknown mapping shows the mapping UI
- [ ] The probe batch is deleted before re-upload
- [ ] An optional origin-account column can be mapped and parsed

## Notes

- Integrates v0.5 amendments: explanatory message on manual mapping; automatic deletion of mapping probes.
- Integrates v0.7 amendment: optional origin-account column feeding the account auto-suggestion (RF-04).
- File name and header signature are distinct identifiers: the file name uniquely identifies a batch (RF-16), while the signature identifies the bank's export layout. Monthly statements from the same bank have different names but the same signature, so the memorized mapping applies automatically.
- The probe flow may re-upload the same file name because the probe batch is deleted before the re-upload (RF-17), which frees the name for RF-16.
- Exactly one mapping is stored per unique signature, enforced at the storage level (TRD section 2).
