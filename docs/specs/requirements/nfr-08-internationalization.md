# NFR-08 - Internationalization

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Category | Internationalization |
| Related | RF-19 (settings host the language switch) |

## Description

The UI is available in **English, French and Spanish**:

- **Default language**: English.
- **Auto-detection**: on first visit, the language is detected from the browser's `Accept-Language` header.
- **Manual override**: the user can change the language in the app settings; the choice persists.
- **Locale formatting**: amounts and dates are formatted according to the selected locale.

## Metric

UI string coverage per locale; correctness of amount/date formatting per locale.

## Threshold

- **100%** of user-facing strings translated in en/fr/es (no fallback keys visible).
- Amounts and dates render per the selected locale's conventions.

## Verification

- i18n unit tests: every key resolves in the three locales (`frontend/src/shared/i18n`).
- Locale formatting tests for money and date utilities (`shared/lib/money`, `shared/lib/dates`).
