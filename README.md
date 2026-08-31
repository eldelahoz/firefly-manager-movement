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
| --- | --- | --- |
| `GET` | `/health` | Health check |
| `GET` | `/api/firefly/accounts` | List FF3 asset accounts (synced to cache) |
| `POST` | `/api/firefly/accounts/{id}/transactions` | Get account transactions for a period |
| `QUERY` | `/api/firefly/accounts/{id}/transactions` | Same as POST (RFC 9295) |

## Documentation

| Document | Content |
| --- | --- |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Local dev setup, environment variables, tests, git workflow, coding conventions |
| [docs/README.md](docs/README.md) | Documentation index - entry point to specs, requirements catalog, and validation matrix |
| [docs/specs/](docs/specs/) | Product requirements (PRD) and technical architecture (TRD) |

### Reading the docs with Obsidian

The documentation (~40 Markdown files: PRD, TRD, 19 functional requirements, 13 non-functional requirements, validation matrix) is read-heavy - dense tables and cross-references between requirements. Opening [`docs/`](docs/README.md) as an [Obsidian](https://obsidian.md) vault improves the reading experience without touching the files: they remain plain Markdown, fully readable on GitHub or VS Code.

The vault ships with curated, committed settings (`docs/.obsidian/`): reading mode by default, readable line width, the Minimal theme, and a small CSS snippet. Volatile files (workspace state, plugin binaries) are git-ignored.

Setup:

1. Open Obsidian → *Open folder as vault* → select `docs/`.
2. If prompted to enable community plugins, accept, then install **Minimal Theme Settings** below (Obsidian downloads it from the community catalog; the binary is not committed).

Curated plugins:

| Plugin | Type | Why |
| --- | --- | --- |
| Minimal | Theme (bundled in `docs/.obsidian/themes/`) | Clean typography; committed with the vault |
| Minimal Theme Settings | Community plugin | Tunable reading: **normal line width**, table density and style - the specs are table-heavy |
| Outline | Core plugin (already enabled) | In-document navigation for the long PRD/TRD files; no install needed |

Entry point: [docs/README.md](docs/README.md) - every RF/NFR is reachable in ≤ 2 clicks.

## License

Private project - all rights reserved.
