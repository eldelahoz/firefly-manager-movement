# Documentation Index

Map of Content for the project documentation. Every requirement, spec, and validation artifact is reachable from this page.

| Document | Content |
| --- | --- |
| [PRD Bank Reconciliation for Firefly III](specs/2026-08-14-prd-rapprochement-bancaire.md) | Product requirements: context, objectives, personas, functional flow, risks |
| [TRD Technical Architecture](specs/2026-08-14-trd-architecture-technique.md) | Architecture: data model, API contract, Firefly III integration, deployment |
| [Validation Matrix](validation-matrix.md) | Maps each RF to the tests that verify it, by layer (domain / app / infra / presentation / frontend) |


## Functional Requirements (RF)

Grouped by priority. Every file follows the same template: field table, description, preconditions, steps, expected result, exceptions, and acceptance criteria.

### P0 required for v1

| ID | Requirement |
| --- | --- |
| [RF-01](specs/requirements/rf-01-upload.md) | CSV Upload |
| [RF-02](specs/requirements/rf-02-csv-mapping.md) | CSV Column Mapping |
| [RF-03](specs/requirements/rf-03-invalid-lines.md) | Invalid Line Reporting |
| [RF-04](specs/requirements/rf-04-asset-accounts.md) | Asset Account Selection |
| [RF-05](specs/requirements/rf-05-fetch-transactions.md) | Target Account Transaction Fetching |
| [RF-06](specs/requirements/rf-06-matching-engine.md) | Matching Engine |
| [RF-07](specs/requirements/rf-07-result-groups.md) | Result Groups |
| [RF-08](specs/requirements/rf-08-batch-creation.md) | Batch Creation of Missing Transactions |
| [RF-14](specs/requirements/rf-14-account-browse.md) | Account Transaction Browsing |
| [RF-15](specs/requirements/rf-15-duplicate-handling.md) | Duplicate Flagging and Deletion |

### P1 required, may follow v1

| ID | Requirement |
| --- | --- |
| [RF-09](specs/requirements/rf-09-batch-history.md) | Batch Immutability and History |
| [RF-10](specs/requirements/rf-10-ignore-movements.md) | Ignore / Un-ignore Movements |
| [RF-11](specs/requirements/rf-11-origin-trace.md) | Origin Trace on Created Transactions |
| [RF-12](specs/requirements/rf-12-currencies.md) | Currency Handling |
| [RF-13](specs/requirements/rf-13-reconciliation-rerun.md) | Reconciliation Re-run |
| [RF-16](specs/requirements/rf-16-duplicate-filename.md) | Duplicate Filename Refusal |
| [RF-17](specs/requirements/rf-17-batch-deletion.md) | Parsed Batch Deletion |
| [RF-18](specs/requirements/rf-18-import-resume.md) | Import Resume |
| [RF-19](specs/requirements/rf-19-connection-settings.md) | Firefly III Connection Settings |

## Non-Functional Requirements (NFR)

Quality attributes; not prioritized. Each file defines description, metric, threshold, and verification.

| ID | Requirement |
| --- | --- |
| [NFR-01](specs/requirements/nfr-01-architecture.md) | Architecture |
| [NFR-02](specs/requirements/nfr-02-security.md) | Security |
| [NFR-03](specs/requirements/nfr-03-privacy.md) | Privacy |
| [NFR-04](specs/requirements/nfr-04-performance.md) | Performance |
| [NFR-05](specs/requirements/nfr-05-idempotency.md) | Idempotency of Creations |
| [NFR-06](specs/requirements/nfr-06-amount-integrity.md) | Amount Integrity |
| [NFR-07](specs/requirements/nfr-07-observability.md) | Observability |
| [NFR-08](specs/requirements/nfr-08-internationalization.md) | Internationalization |
| [NFR-09](specs/requirements/nfr-09-error-handling.md) | Error Handling |
| [NFR-10](specs/requirements/nfr-10-data-retention.md) | Data Retention |
| [NFR-11](specs/requirements/nfr-11-accessibility.md) | Accessibility |
| [NFR-12](specs/requirements/nfr-12-responsive.md) | Responsive Layout |
| [NFR-13](specs/requirements/nfr-13-browser-support.md) | Browser Support |

## Reading this documentation

All files are plain Markdown and render on GitHub or VS Code. For a better reading experience, open `docs/` as an Obsidian vault see the [repository README](../README.md#documentation) for setup and the curated plugin list.
