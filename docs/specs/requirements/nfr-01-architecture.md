# NFR-01 - Architecture

| Field | Value |
| --- | --- |
| Category | Architecture |
| Related | TRD section 1, RF-19 |

## Description

Standalone web application composed of a Rust backend, a React frontend and a MySQL database. Deployable with Docker Compose on a shared network with the existing Firefly III instance. Database schema migrations run automatically at backend startup.

## Metric

Composition of the deployment (components and technologies); automatic migration execution.

## Threshold

- All application components (backend, frontend, database) start as one stack; the external FF3 instance is reachable over the shared network.
- `docker compose up` yields a fully working stack (migrations included).

## Verification

- Deploy from a clean environment with `docker compose up`; assert all services reach healthy state and the backend responds after migrations complete.
