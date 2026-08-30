# firefly-manager-movement

![Rust](https://img.shields.io/badge/rust-1.75+-orange?logo=rust)
![Axum](https://img.shields.io/badge/axum-0.8-blue?logo=rust)
![MySQL](https://img.shields.io/badge/mysql-9.7-4479a1?logo=mysql)
![Architecture](https://img.shields.io/badge/clean--architecture-onion-lightgrey)
![Status](https://img.shields.io/badge/status-in%20development-yellow)

Bank reconciliation (rapprochement bancaire) backend imports financial movements from CSV sources and syncs them with [Firefly III](https://www.firefly-iii.org/).

Built in Rust with **Clean Architecture** (domain → application → infrastructure → presentation).

## Project Structure

```
backend/
├── domain/           # Entities, value objects, matching rules
├── application/      # Use cases, ports (interfaces), DTOs, errors
├── infrastructure/   # MySQL (diesel), Firefly III HTTP client, repositories, migrations/
├── presentation/     # HTTP API (axum), OpenAPI (utoipa + Scalar)
├── Cargo.toml        # Workspace root
└── diesel.toml       # Diesel CLI config
```

## Architecture

```
┌─────────────────────────────────────────┐
│         Presentation (axum)             │
│  HTTP handlers, OpenAPI (utoipa)        │
├─────────────────────────────────────────┤
│       Application (use cases)           │
│  Ports, DTOs, business orchestration    │
├─────────────────────────────────────────┤
│       Infrastructure (adapters)         │
│  MySQL (diesel), Firefly III client     │
├─────────────────────────────────────────┤
│         Domain (core)                   │
│  Entities, value objects, rules         │
└─────────────────────────────────────────┘
     Dependencies point inward →
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/api/firefly/accounts` | List FF3 asset accounts (synced to cache) |
| `POST` | `/api/firefly/accounts/{id}/transactions` | Get account transactions for a period |
| `QUERY` | `/api/firefly/accounts/{id}/transactions` | Same as POST (RFC 9295) |

## Documentation

| Document | Content |
|----------|---------|
| [COLLABORATION.md](COLLABORATION.md) | Local dev setup, environment variables, tests, git workflow, coding conventions |
| [docs/specs/](docs/specs/) | Product requirements (PRD) and technical architecture (TRD) |

## License

Private project — all rights reserved.
