# NFR-02 - Security

| Field | Value |
| --- | --- |
| Version | v0.6 (integrates v0.6 amendment: connection moved from env vars to app settings) |
| Category | Security |
| Related | RF-19, TRD section 4 |

## Description

The Firefly III connection (URL + API token) is configured in the app settings (RF-19) and stored **backend-side only** - it is never exposed to the browser. After saving, the token is never returned by any API response. No banking data (statements, movements) is sent to any third-party service; all traffic stays between the app and the configured Firefly III instance.

## Metric

Occurrences of the API token in frontend-visible payloads/responses; count of outbound connections to hosts other than the configured FF3 instance and the database.

## Threshold

- **0** token occurrences in any frontend-reachable response.
- **0** outbound connections to third-party hosts.

## Verification

- UI check: the token is not returned after save.
- Code review: the token is sourced from backend-side configuration only.
- Code review of API responses (no token field in DTOs).
