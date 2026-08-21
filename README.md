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
├── crates/
│   ├── domain/           # Entities, value objects, matching rules
│   ├── application/      # Use cases, ports (interfaces), DTOs, errors
│   ├── infrastructure/   # MySQL (diesel), Firefly III HTTP client, repositories
│   └── presentation/     # HTTP API (axum), OpenAPI (utoipa + Scalar)
├── migrations/           # Diesel database migrations
├── Cargo.toml            # Workspace root
└── diesel.toml           # Diesel CLI config
```

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.75+ | [rustup.rs](https://rustup.rs) |
| Diesel CLI | latest | `cargo install diesel_cli --no-default-features --features mysql` |
| Docker | 24+ | [docker.com](https://www.docker.com/products/docker-desktop/) |

## Quick Start

### 1. Start Dev Infrastructure (MySQL + Adminer)

```bash
docker compose -f docker-compose.dev.yml up -d
```

This starts MySQL on `localhost:3306` and Adminer on `localhost:8080`.

### 2. Configure Environment

```bash
cd backend
cp .env.example .env
# Edit .env if you need custom values
```

### 3. Apply Database Migrations

```bash
cd backend
diesel migration run
```

### 4. Run the Backend

```bash
cd backend
cargo run --bin ffm-backend
```

The server starts on **http://localhost:3000**.

### 5. Open API Docs

- **Scalar UI**: http://localhost:3000/scalar
- **OpenAPI JSON**: http://localhost:3000/api-docs/openapi.json

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | MySQL connection string | `mysql://ffm:devffmpass@localhost:3306/firefly_manager` |
| `FF3_URL` | Firefly III base URL | `http://localhost:8080` |
| `FF3_TOKEN` | Firefly III Bearer token | *(required)* |
| `MATCH_TOLERANCE_DAYS` | Days tolerance for transaction matching | `0` |
| `RUST_LOG` | Log level filter | `ffm_presentation=info,...` |
| `FF3_HEALTH_PROBE` | Enable Firefly III probe in `/health` | `false` |

## Running Tests

```bash
# All workspace tests (requires MySQL running)
cd backend
cargo test --workspace

# Specific crate
cargo test -p ffm-domain
cargo test -p ffm-application
cargo test -p ffm-infrastructure
cargo test -p ffm-presentation
```

## Useful Commands

| Command | Description |
|---------|-------------|
| `cargo fmt --all` | Format code |
| `cargo clippy --workspace --all-targets -- -D warnings` | Lint |
| `cargo test --workspace` | Run all tests |
| `diesel migration run` | Apply migrations |
| `diesel migration redo` | Rollback + reapply last migration |
| `docker compose -f docker-compose.dev.yml down -v` | Stop + remove volumes |

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

## License

Private project — all rights reserved.
