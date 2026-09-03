# NFR-03 - Privacy

| Field | Value |
| --- | --- |
| Version | Original (2026-08-14) — unamended |
| Category | Privacy |
| Related | NFR-02 |

## Description

The application is **self-hosted only**. It MUST NOT collect or transmit telemetry, analytics or crash reports. The only outbound network calls are: to the configured Firefly III instance and to the MySQL database.

## Metric

Count of telemetry/analytics SDKs in the dependency tree; count of outbound requests to hosts other than FF3/MySQL in the test suite.

## Threshold

- **0** telemetry or analytics dependencies (frontend and backend).
- **0** outbound requests to non-configured hosts.

## Verification

- Dependency audit (`pnpm` license/dependency list, `cargo tree`) shows no telemetry SDKs.
- Network inspection of the frontend test suite (MSW handlers) records no external calls.
