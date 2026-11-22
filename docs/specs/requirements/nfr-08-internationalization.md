# NFR-08 - Internationalization

| Field | Value |
| --- | --- |
| Version | v0.1 (integrates v0.1 amendment: number format preference) |
| Category | Internationalization |
| Related | RF-19 (settings host the language switch and the number format), TRD section 1 |

## Description

The UI is available in **English, French and Spanish**:

- **Default language**: English.
- **Auto-detection**: on first visit, the language is detected from the browser's `Accept-Language` header.
- **Manual override**: the user can change the language in the app settings; the choice persists.
- **Locale formatting**: dates are formatted according to the selected locale. Amounts follow the **number format** preference (below).

### Number format (v0.1)

The user can choose how amounts are written (thousands and decimal separators) in the app settings, next to the language. The language alone is not enough: es-MX and es-CO share a language but not a number format.

| Value | Example | Representative locale |
| --- | --- | --- |
| `auto` (default) | follows the UI language | the selected UI language |
| `comma-dot` | 1,234,567.89 | en-US |
| `dot-comma` | 1.234.567,89 | de-DE |
| `space-comma` | 1 234 567,89 (narrow no-break space, U+202F) | fr-FR |
| `apostrophe-dot` | 1'234'567.89 | de-CH |
| `comma-dot-in` | 12,34,567.89 | en-IN |

- **Default**: `auto`, which keeps today's behavior (the format of the UI language).
- **Storage and effect**: client-only, stored in the browser (`localStorage`) next to the language and applied immediately, with no Save button. Nothing is sent to the backend.
- **Grouping and decimals**: thousands are always grouped (a 4-digit integer part is grouped too) and amounts always show exactly 2 decimals, since amounts are stored as cents (NFR-06). The currency unit follows the amount as before (CLDR symbol for the UI language, e.g. EUR → €, COP → COP) and the sign stays explicit.
- **Visual clarity**: the decimal separator and the cents are rendered muted (secondary color) with tabular figures; the text content of the amount does not change.
- **Out of scope**: dates (unchanged), the decimal separator of CSV statements, zero-decimal currencies.

## Metric

UI string coverage per locale; correctness of amount/date formatting per locale and per number format.

## Threshold

- **100%** of user-facing strings translated in en/fr/es (no fallback keys visible).
- Dates render per the selected locale's conventions.
- Amounts render per the selected number format, with the acceptance criteria below.

## Acceptance criteria (v0.1)

1. **Given** the settings page, **when** the user opens the number format select, **then** it offers exactly the 6 values above, each option labelled with its example (not a name), and a live preview of an amount in the current choice.
2. **Given** no stored preference, **when** the app loads, **then** the format is `auto`, and amounts match the UI language (e.g. `1.234,56` in es, `1,234.56` in en).
3. **Given** a format other than `auto`, **when** the user selects it, **then** every displayed amount (dashboard, results, summary) changes immediately without reload or Save, and the UI language is unchanged.
4. **Given** a selected format, **when** the page is reloaded, **then** the choice is kept (`localStorage` key `ffm.numberFormat`; the language key is `ffm.locale`).
5. **Given** the amount 1234.56 (123456 cents) in each of the 5 explicit formats, **then** it renders as `1,234.56`, `1.234,56`, `1 234,56` (U+202F), `1'234.56` and `1,234.56`; and 1234567.89 renders as the table examples above, including `12,34,567.89` for `comma-dot-in`.
6. **Given** any format, **then** a 4-digit integer part is grouped, and an amount in whole units shows exactly 2 decimals (`1,000.00`), followed by the currency unit as before (CLDR symbol for the UI language: EUR → €, COP → COP), independent of the number format.
7. **Given** an amount, **then** the decimal separator and the cents are rendered muted (secondary color) and the element's text content is the same as without styling, for amounts shown with `MoneyText` (table rows, Browse, the settings preview); amounts embedded in translated sentences (e.g. the Results summary) are plain text and are not muted.
8. **Given** the number format changes, **then** no request is made to the backend and dates keep their formatting.

## Verification

- i18n unit tests: every key resolves in the three locales (`frontend/src/shared/i18n`).
- Locale formatting tests for the date utilities (`shared/lib/dates`).
- Playwright (`frontend/tests/specs/**`): one test per number format and for `auto`, 4-digit grouping, muted fraction, persistence after reload, immediate effect, no backend call (criteria 1-8).

## Notes

- v0.1 amendment (number format): adds the client-only number format preference (6 values, `auto` default) and replaces "amounts and dates are formatted according to the selected locale" with: dates follow the locale, amounts follow the number format. No change to language selection or detection. Decisions: `docs/superpowers/plans/2026-11-22-number-format.md`.
