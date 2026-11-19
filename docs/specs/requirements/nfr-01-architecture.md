# NFR-01 - Architecture

| Field | Value |
| --- | --- |
| Version | v0.11 (integrates PRD v0.11 amendment) |
| Category | Architecture |
| Related | TRD section 1, RF-19 |

## Description

Standalone web application composed of a Rust backend, a React frontend and an embedded SQLite database, shipped as **one Docker image**: the backend serves the API and the built frontend on the same port, and the database is a single file on a mounted volume. The existing Firefly III instance is external, reached by URL + token (RF-19). Database schema migrations run automatically at backend startup.

> **Amendment (v0.11, 2026-11-18)**: MySQL replaced by SQLite; the three-service stack (backend, frontend, database) replaced by one container. Previously: "Rust backend, React frontend and a MySQL database [...] on a shared network with the existing Firefly III instance".

## Metric

Composition of the deployment (components and technologies); automatic migration execution.

## Threshold

- The whole application (API, UI, database) runs in **one container**; no separate database service is required.
- `docker compose up` yields a fully working application (migrations included) with its data persisted on a volume across restarts.
- The external FF3 instance is reachable from the container by its configured URL.

## Verification

- Build and start from a clean environment with `docker compose up`; assert the container serves `/health` and the UI on the same port after migrations complete.
- Restart the container; assert previously imported batches are still listed (volume persistence).
