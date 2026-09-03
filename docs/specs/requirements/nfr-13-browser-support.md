# NFR-13 - Browser Support

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Category | Compatibility |
| Related | Frontend build configuration |

## Description

The supported browsers are the **last 2 major versions** of Chrome, Firefox, Safari and Edge. Internet Explorer 11 and legacy browsers are explicitly **not supported** - no compatibility shims are provided.

## Metric

Build target configuration (browserslist / Vite targets) versus the support matrix.

## Threshold

Build targets cover exactly the supported set; CI builds successfully against those targets.

## Verification

- `browserslist` configuration in the frontend lists the supported browsers (last 2 versions of the four engines, no IE).
- `pnpm build` succeeds with those targets in CI.
