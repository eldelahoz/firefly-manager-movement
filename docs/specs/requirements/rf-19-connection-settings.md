# RF-19 - Firefly III Connection Settings

| Field | Value |
| --- | --- |
| Priority | P1 |
| Use cases | CU-11 |
| Depends on | - (prerequisite for every FF3-touching requirement) |
| Related | NFR-02 (token security), RF-04, RF-05, RF-08 (default creation tag), RF-11, RF-14 |

## Description

The app settings MUST allow configuring the Firefly III connection:

- **Fields**: Firefly III URL and API token.
- **Default creation tag**: an optional tag picked from a dropdown listing the existing Firefly III tags; it is applied by default to the transactions created through the app (RF-08), where the user's selection replaces it. An empty value means no default tag.
- **Connectivity check**: a "verify connection" action probes the Firefly III instance with the entered credentials and reports success/failure before saving.
- **Persistence**: the connection is saved app-side and **replaces environment-variable configuration**; every FF3-dependent feature uses it.
- **Required before first use**: any feature needing Firefly III (accounts, reconciliation, browsing) requires a configured connection.
- **Security**: the token is stored backend-side only and is never returned to the frontend after saving (NFR-02).

## Preconditions

1. The user has access to the app settings page.

## Steps

1. **Given** the settings page,
2. **When** the user enters the FF3 URL and API token,
3. **And** triggers the connectivity check,
4. **Then** the system probes the instance and reports the result.
5. **When** the user saves,
6. **Then** the connection is persisted backend-side and used for all subsequent FF3 calls.

## Expected result

- With a configured, valid connection, all FF3-dependent features work; the token never transits back to the browser.

## Exceptions

- **Invalid URL or token**: the connectivity check fails with a clear message; nothing is saved.
- **FF3 unreachable during check**: explicit failure with retry (NFR-09).
- **Using FF3-dependent features without a configured connection**: the app directs the user to the settings page.

## Acceptance criteria

- [ ] The connectivity check validates the URL and token against the instance
- [ ] The settings form: URL + token, "Test connection" button, save persists
- [ ] The token is not exposed to the frontend after save
- [ ] The default-creation-tag dropdown lists existing FF3 tags; empty is allowed and persists

## Notes

- Integrates v0.6 amendment. Settings storage and related interfaces are specified in the TRD (section 3).
- Integrates v0.7 amendment: default creation tag (optional, FF3-sourced dropdown) pre-filling the tags field at creation (RF-08).
