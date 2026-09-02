# NFR-11 - Accessibility

| Field | Value |
| --- | --- |
| Category | Accessibility |
| Related | NFR-12 (responsive) |

## Description

The UI conforms to **WCAG 2.1 level AA** at minimum:

- All interactive elements (buttons, checkboxes, selects, dialogs, tabs) are reachable and operable **by keyboard alone**, with a visible focus state.
- Images and meaningful icons carry text alternatives (alt text / accessible labels); purely decorative elements are hidden from assistive technology.
- Dialogs (delete confirmation, edit forms) trap focus and close predictably.
- Color is never the sole carrier of information (gap highlighting pairs color with badges/text).

## Metric

Automated accessibility violations (axe ruleset) on the main pages; keyboard-only operability of the critical flows.

## Threshold

- **0** serious/critical axe violations on Dashboard, Import wizard, Results, Settings.
- Critical flows (upload → reconcile → apply, duplicate deletion) completable keyboard-only.

## Verification

- Accessibility audit (axe) integrated in the frontend test suite on the main pages.
- Keyboard navigation tests for wizard steps, results tabs and the delete dialog.
