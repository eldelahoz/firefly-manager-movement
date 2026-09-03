# NFR-12 - Responsive Layout

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Category | UX |
| Related | NFR-11 |

## Description

- The interface is responsive and fully usable on screens of **1024 px and wider** (primary target: desktop).
- The import wizard specifically remains usable on tablets down to **768 px** wide.

## Metric

Rendering and operability of the main pages at viewport widths 768 px, 1024 px, 1440 px, 1920 px.

## Threshold

- No horizontal scrolling, overlapping or clipped interactive elements at any tested width ≥ 1024 px (≥ 768 px for the wizard).
- All critical flows operable at the supported widths.

## Verification

- Viewport-based component/render tests (Vitest) at 768/1024/1440/1920 px.
- Manual device-matrix check on the wizard (upload, mapping, reconcile steps).
