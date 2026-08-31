# Collaboration Guide

How to set up a local development environment and contribute to this repo.

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.75+ | [rustup.rs](https://rustup.rs) |
| Diesel CLI | latest | `cargo install diesel_cli --no-default-features --features mysql` |
| Docker | 24+ | [docker.com](https://www.docker.com/products/docker-desktop/) |
| Node.js | 22 LTS | [nodejs.org](https://nodejs.org) |
| pnpm | 10+ | `corepack enable` or [pnpm.io](https://pnpm.io/installation) |

## Services Overview

| Service | Container | Port | Purpose |
|---------|-----------|------|---------|
| MySQL 9.7 | `ffm-mysql` | `3306` | App DB (`firefly_manager`) + Firefly III DB (`fireflyiii`) |
| Adminer | `ffm-adminer` | `8081` | Web database inspector |
| Firefly III | `ffm-fireflyiii` | `8080` | Financial app the backend syncs to |
| Backend (native) | — | `3000` | Rust API server (`cargo run`) |
| Frontend (native) | — | `5173` | React SPA dev server (`pnpm dev`, proxies `/api` + `/health` → `:3000`) |

## Local Dev Environment Setup

### 1. Generate Firefly III APP_KEY

Firefly III needs an `APP_KEY` — a random secret that encrypts sensitive data.
It must not be committed (the root `.env` file is gitignored).

**Linux / WSL / macOS:**

```bash
echo "FF3_APP_KEY=base64:$(openssl rand -base64 32)" > .env
```

**Windows (PowerShell):**

```powershell
$b = New-Object byte[] 32
[System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($b)
"FF3_APP_KEY=base64:$([Convert]::ToBase64String($b))" | Set-Content .env
```

> **Do not change the `APP_KEY` after Firefly III has data.** It encrypts
> sensitive fields — losing it makes existing data unreadable.

### 2. Start Docker Stacks

Start order matters — MySQL must be healthy before Firefly III connects:

```bash
# Step 2a: MySQL + Adminer
docker compose -f docker-compose.dev.yml up -d

# Step 2b: Firefly III (after MySQL is healthy)
docker compose -f docker-compose.firefly.yml up -d
```

Wait ~30s for Firefly III to run its first-time migrations, then verify:

> **Existing MySQL volume?** If `ffm-mysql-data` was created before the Firefly III init script was added, the `fireflyiii` database won't exist. Create it once:
>
> ```bash
> docker exec -i ffm-mysql mysql -uroot -pdevrootpass -e "CREATE DATABASE IF NOT EXISTS fireflyiii CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci; GRANT ALL PRIVILEGES ON fireflyiii.* TO 'ffm'@'%'; FLUSH PRIVILEGES;"
> ```
>
> Then restart Firefly III: `docker restart ffm-fireflyiii`

### 3. Create Firefly III Admin Account

1. Open **http://localhost:8080** in your browser
2. Click **"Register a new account"**
3. Enter an email and password (local dev instance — use any credentials)
4. Check **"Verify password security"**
5. Click **Register**
6. Complete the setup wizard (set currency, skip optional steps)

### 4. Generate Firefly III API Token

The backend needs a Bearer token to communicate with Firefly III:

1. In Firefly III, click **Options** (gear icon, top-right)
2. Go to **Profile** → scroll to **OAuth**
3. Under **Personal Access Tokens** → **Create New Token**
4. Name it (e.g., `ffm-backend-dev`)
5. **Copy the token immediately** — it's only shown once

### 5. Configure Backend Environment

```bash
cd backend
cp .env.example .env
```

Edit `backend/.env` — replace the placeholder token with the one from step 4:

```dotenv
FF3_TOKEN=eyJ0eXAiOiJKV1QiLCJhbG...your-token-here...
```

Optionally enable the FF3 connectivity probe in `/health`:

```dotenv
FF3_HEALTH_PROBE=true
```

### 6. Apply Database Migrations

```bash
cd backend
diesel migration run
```

This creates the `firefly_manager` schema: `ff3_accounts`, `batches`, `movements`, `reconciliations`, `matches`, `creations`, `ff3_transactions`.

### 7. Run the Backend

```bash
cd backend
cargo run --bin ffm-backend
```

The server starts on **http://localhost:3000**.

### 8. Verify Everything Works

```bash
# Backend health (includes FF3 probe if FF3_HEALTH_PROBE=true)
curl http://localhost:3000/health

# List FF3 asset accounts (requires valid FF3_TOKEN)
curl http://localhost:3000/api/firefly/accounts
```

### 9. Open API Docs

- **Scalar UI**: http://localhost:3000/scalar
- **OpenAPI JSON**: http://localhost:3000/api-docs/openapi.json

## Frontend Development

The SPA lives in `frontend/` (React 19 + Vite + TypeScript, Tailwind v4).
It reaches the backend through the Vite dev proxy, so start the backend first (step 7), then:

```bash
cd frontend
pnpm install   # first time only
pnpm dev       # http://localhost:5173 — /api and /health are proxied to localhost:3000
```

No extra env setup is needed for local dev; the SPA uses same-origin requests via the proxy.
Unit and component tests run against MSW mocks — they do not require the backend or Docker.

### Frontend Commands

| Command | Description |
|---------|-------------|
| `pnpm dev` | Dev server on `http://localhost:5173` with API proxy |
| `pnpm test` | Vitest unit/component tests (MSW-mocked API) |
| `pnpm lint` | ESLint + Prettier check |
| `pnpm build` | Type-check (`tsc -b`) + production bundle in `dist/` |
| `pnpm preview` | Serve the production build locally |
| `pnpm api:types` | Regenerate TS types from the committed `openapi.json` snapshot |

### OpenAPI Types

API types are generated from the `frontend/openapi.json` snapshot of the backend spec.
Never edit `src/shared/api/types.ts` by hand. If the backend contract changes,
refresh the snapshot and regenerate:

```bash
# from the repo root, with the backend running
curl http://localhost:3000/api-docs/openapi.json > frontend/openapi.json
cd frontend && pnpm api:types
```

## Stopping Services

```bash
# Stop backend: Ctrl+C in the cargo run terminal

# Stop Docker stacks (keeps data)
docker compose -f docker-compose.firefly.yml down
docker compose -f docker-compose.dev.yml down

# Stop + wipe all data (fresh start next time)
docker compose -f docker-compose.dev.yml down -v
docker compose -f docker-compose.firefly.yml down -v
```

## Environment Variables

| Variable | Location | Description | Default |
|----------|----------|-------------|---------|
| `FF3_APP_KEY` | root `.env` | Firefly III encryption key | *(required — see step 1)* |
| `DATABASE_URL` | `backend/.env` | MySQL connection string | `mysql://ffm:devffmpass@localhost:3306/firefly_manager` |
| `FF3_URL` | `backend/.env` | Firefly III base URL | `http://localhost:8080` |
| `FF3_TOKEN` | `backend/.env` | Firefly III Bearer token | *(required — see step 4)* |
| `MATCH_TOLERANCE_DAYS` | `backend/.env` | Days tolerance for transaction matching | `0` |
| `RUST_LOG` | `backend/.env` | Log level filter | `ffm_presentation=info,...` |
| `FF3_HEALTH_PROBE` | `backend/.env` | Enable Firefly III probe in `/health` | `false` |

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
| `docker compose -f docker-compose.dev.yml down -v` | Stop dev stack + wipe volumes |
| `docker compose -f docker-compose.firefly.yml down -v` | Stop Firefly III + wipe volumes |

## Git Workflow

### Branch Naming

```
<type>/<scope>-<short-description>
```

Examples: `feat/domain-matching-engine`, `fix/infra-pool-leak`, `docs/prd-update`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

| Type | Use for |
|------|---------|
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `refactor` | Code restructuring without behavior change |
| `test` | Adding or fixing tests |
| `chore` | Tooling, config, dependencies |

Scopes mirror the crate structure: `domain`, `application`, `infra`, `presentation`, `backend`, `frontend`.

Examples from this repo:
- `feat(domain): MatchingEngine — apareamiento 1-a-1 por monto+sens`
- `feat(infra): ReqwestFireflyClient with exhaustive pagination`
- `docs(backend): translate code comments to English`

### Before Pushing

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

All three must pass. Clippy warnings are treated as errors (`-D warnings`).

For changes under `frontend/`, the equivalent gate must also pass:

```bash
cd frontend
pnpm lint && pnpm test && pnpm build
```

## Coding Conventions

- **Clean Architecture**: dependencies point inward only (presentation → application → domain). Domain has zero dependencies on outer layers.
- **Language**: code comments in English, PRD/spec docs may be in French (business domain language).
- **No comments unless necessary** — code should be self-documenting.
- **Diesel migrations**: one migration per logical change, always include `up.sql` and `down.sql`.
- **New repository?** Add it to `backend/infrastructure/src/persistence/repositories/mod.rs` and implement the corresponding port trait from `backend/application/src/ports/`.

## Reading the Docs (Obsidian Vault)

`docs/` doubles as an [Obsidian](https://obsidian.md) vault: *Open folder as vault* → select `docs/`.
It ships a curated, committed configuration — Minimal theme (vendored, MIT © Steph Ango),
reading view by default, readable line length (~704px).

- **Committed (curated):** `docs/.obsidian/app.json`, `appearance.json`, `core-plugins.json`, `snippets/docs-reading.css`, `themes/Minimal/` (`theme.css`, `manifest.json`, `LICENSE`).
- **Gitignored (volatile/personal):** everything else under `docs/.obsidian/` — `workspace.json`, `workspace-mobile.json`, plugins, caches, and any themes/snippets you add locally.
- The docs remain plain Markdown: GitHub/VSCode rendering is unaffected, and agents (AGENTS.md) do not depend on Obsidian.
