# Collaboration Guide

How to set up a local development environment and contribute to this repo.

## Prerequisites

| Tool | Version | Install |
| --- | --- | --- |
| Rust | 1.75+ | [rustup.rs](https://rustup.rs) |
| Diesel CLI | latest | `cargo install diesel_cli --no-default-features --features mysql` |
| Docker | 24+ | [docker.com](https://www.docker.com/products/docker-desktop/) |
| Node.js | 22 LTS | [nodejs.org](https://nodejs.org) |
| pnpm | 10+ | `corepack enable` or [pnpm.io](https://pnpm.io/installation) |

## Services Overview

| Service | Container | Port | Purpose |
| --- | --- | --- | --- |
| MySQL 9.7 | `ffm-mysql` | `3306` | App DB (`firefly_manager`) + Firefly III DB (`fireflyiii`) |
| Adminer | `ffm-adminer` | `8081` | Web database inspector |
| Firefly III | `ffm-fireflyiii` | `8080` | Financial app the backend syncs to |
| Backend (native) | - | `3000` | Rust API server (`cargo run`) |
| Frontend (native) | - | `5173` | React SPA dev server (`pnpm dev`, proxies `/api` + `/health` → `:3000`) |

## Local Dev Environment Setup

### 1. Generate Firefly III APP_KEY

Firefly III needs an `APP_KEY` - a random secret that encrypts sensitive data.
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
> sensitive fields - losing it makes existing data unreadable.

### 2. Start Docker Stacks

Start order matters - MySQL must be healthy before Firefly III connects:

```bash
# Step 2a: MySQL + Adminer
docker compose -p ffm-dev -f docker-compose.dev.yml up -d

# Step 2b: Firefly III (after MySQL is healthy)
docker compose -p ffm-firefly -f docker-compose.firefly.yml up -d
```
> Both compose files run from the same directory, so Docker Compose defaults to the same project name (`firefly-manager-movement`). Without explicit project names, the second `up` sees containers from the first file that aren't in its own file → "orphan containers" warning. The `-p` flag isolates them into separate projects.

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
3. Enter an email and password (local dev instance - use any credentials)
4. Check **"Verify password security"**
5. Click **Register**
6. Complete the setup wizard (set currency, skip optional steps)

### 4. Generate Firefly III API Token

The backend needs a Bearer token to communicate with Firefly III:

1. In Firefly III, click **Options** (gear icon, top-right)
2. Go to **Profile** → scroll to **OAuth**
3. Under **Personal Access Tokens** → **Create New Token**
4. Name it (e.g., `ffm-backend-dev`)
5. **Copy the token immediately** - it's only shown once

### 5. Configure Backend Environment

```bash
cd backend
cp .env.example .env
```

Edit `backend/.env` - replace the placeholder token with the one from step 4:

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
pnpm dev       # http://localhost:5173 - /api and /health are proxied to localhost:3000
```

No extra env setup is needed for local dev; the SPA uses same-origin requests via the proxy.
Unit and component tests run against MSW mocks - they do not require the backend or Docker.

### Frontend Commands

| Command | Description |
| --- | --- |
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
docker compose -p ffm-firefly -f docker-compose.firefly.yml down
docker compose -p ffm-dev -f docker-compose.dev.yml down

# Stop + wipe all data (fresh start next time)
docker compose -p ffm-dev -f docker-compose.dev.yml down -v
docker compose -p ffm-firefly -f docker-compose.firefly.yml down -v
```

## Environment Variables

| Variable | Location | Description | Default |
| --- | --- | --- | --- |
| `FF3_APP_KEY` | root `.env` | Firefly III encryption key | *(required - see step 1)* |
| `DATABASE_URL` | `backend/.env` | MySQL connection string | `mysql://ffm:devffmpass@localhost:3306/firefly_manager` |
| `FF3_URL` | `backend/.env` | Firefly III base URL | `http://localhost:8080` |
| `FF3_TOKEN` | `backend/.env` | Firefly III Bearer token | *(required - see step 4)* |
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
| --- | --- |
| `cargo fmt --all` | Format code |
| `cargo clippy --workspace --all-targets -- -D warnings` | Lint |
| `cargo test --workspace` | Run all tests |
| `diesel migration run` | Apply migrations |
| `diesel migration redo` | Rollback + reapply last migration |
| `docker compose -p ffm-dev -f docker-compose.dev.yml down -v` | Stop dev stack + wipe volumes |
| `docker compose -p ffm-firefly -f docker-compose.firefly.yml down -v` | Stop Firefly III + wipe volumes |

## Git Workflow

All commit messages, PR titles, PR descriptions, and tooling output are written in **English**. (PRD/spec docs may stay in French; code comments follow the Coding Conventions section below.)

### Branch Naming

```
<type>/<scope>-<short-description>
```

Examples: `feat/domain-matching-engine`, `fix/infra-pool-leak`, `docs/prd-update`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/). The format is enforced locally by `commitlint` + `lefthook` (installed via `pnpm install` at the repo root) and re-validated in CI:

```
<type>(<scope>): <subject>

[body - explains WHAT and WHY, wrapped at <= 100 chars per line]

Refs: RF-XX, NFR-YY
```

| Type | Use for |
| --- | --- |
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `refactor` | Code restructuring without behavior change |
| `test` | Adding or fixing tests |
| `chore` | Tooling, config, dependencies |
| `ci` | GitHub Actions, hooks, commit-convention tooling |

| Scope | Targets |
| --- | --- |
| `domain` `application` `infra` `presentation` `backend` `frontend` | The Clean Architecture layers / crates / app |
| `matrix` `specs` `docs` | `docs/validation-matrix.md`, `docs/specs/`, other repo docs |
| `deps` `ci` `docker` | Lockfiles/dependencies, workflows/hooks tooling, compose stacks |

Rules (hard failures unless noted):

- **Header (the whole first line) <= 100 chars**; >= 72 emits a warning - push details into the body.
- **One scope per commit** (`docs(a,b): ...` is rejected; the body carries the detail).
- **Subject in English, imperative mood, no trailing period** (`feat(backend): add ...`, not `feat(backend): Añade ...` or `feat(backend): added.`). The hook rejects obvious Spanish residues (letters like `ñ á é í ó ú ü ¿ ¡` and function words `con|por|para|sin|...`); it does not catch fluent Spanglish - reviewers are the last line for wording quality.
- **`Refs: RF-XX, NFR-YY` footer is required** whenever the commit touches `docs/specs/requirements/**` (checked by the `commit-msg` hook). For implementation changes covered by a requirement, the PR checklist carries the same duty.

Valid / invalid (real history rewrites):

| Invalid | Valid |
| --- | --- |
| `feat(domain): MatchingEngine - apareamiento 1-a-1 por monto+sens` | `feat(domain): 1-to-1 matching by amount+sensitivity` |
| `feat(infra): MysqlBatchRepository + MysqlMovementRepository con tests de integracion` | `feat(infra): MySQL batch and movement repositories with integration tests` |
| `feat(infra): repos MySQL restantes (Ff3Account upsert, Reconciliation, ...) + tests integracion` (149 chars) | `feat(infra): remaining MySQL repositories` (list them in the body) |
| `docs(domain,application,presentation): normalize date-format doc comments` | `docs(backend): normalize date-format doc comments across crates` |
| `chg: ...` / `add: ...` / `format(frontend): ...` | map to `chore` / `feat` / `chore(frontend)` |

### Pull Requests

Every new PR is pre-filled with `.github/PULL_REQUEST_TEMPLATE.md` (English). The PR title follows the same convention as commits (`<type>(<scope>): <subject>`, <= 100 chars, English) and is linted in CI (`lint-pr-title` job). The head branch is deleted automatically when the PR merges (repo setting `delete_branch_on_merge`; for `gh pr merge` pass `--delete-branch` to be explicit).

**Merge style: squash only.** Merge commits and rebase merges are disabled repo-wide (`gh pr merge --squash`, or the UI default). The resulting commit on `main` is `<PR title> (#NN)` - so every commit visible in `git log main` is a conventional, linted one-liner that references its PR.

No direct pushes to `main`, enforced in layers:

1. **GitHub branch protection** (`main`: require a PR + the `lint-commits` / `lint-pr-title` / `guard-main` status checks). *Not yet active:* the repository is on the free private plan, where the protection/ruleset APIs return 403 - apply via `PUT /repos/eldelahoz/firefly-manager-movement/branches/main/protection` once the plan allows it (required contexts: `lint-commits`, `lint-pr-title`, `guard-main`; `allowed_merge_methods: ["squash"]`). Until then, layers 2 and 3 are the enforcement.
2. **Local pre-push hook** (`lefthook`): `git push origin main` is rejected with an English message pointing at `git switch -c <type>/<scope>-<desc> && git push -u origin HEAD && gh pr create`.
3. **CI `guard-main` job**: the head commit of any push to `main` must end with the PR marker ` (#NN)` (produced by the squash merge UI); otherwise the pipeline fails.

`--no-verify` is banned except for documented hotfixes (justify in the PR template section); CI and protection still catch the violation.

### Before Pushing

```bash
# from the repo root (one-time: pnpm install to sync the git hooks)
pnpm exec commitlint --from=HEAD~1

# backend (via WSL)
cd backend
wsl cargo fmt --all
wsl cargo clippy --workspace --all-targets -- -D warnings
wsl cargo test --workspace
```

All gates must pass. Clippy warnings are treated as errors (`-D warnings`).

For changes under `frontend/`, the equivalent gate must also pass:

```bash
cd frontend
pnpm lint && pnpm build && pnpm e2e
```

## Coding Conventions

- **Clean Architecture**: dependencies point inward only (presentation → application → domain). Domain has zero dependencies on outer layers.
- **Language**: code comments in English, PRD/spec docs may be in French (business domain language).
- **No comments unless necessary** - code should be self-documenting.
- **Diesel migrations**: one migration per logical change, always include `up.sql` and `down.sql`.
- **New repository?** Add it to `backend/infrastructure/src/persistence/repositories/mod.rs` and implement the corresponding port trait from `backend/application/src/ports/`.

## Reading the Docs (Obsidian Vault)

`docs/` doubles as an [Obsidian](https://obsidian.md) vault: *Open folder as vault* → select `docs/`.
It ships a curated, committed configuration - Minimal theme (vendored, MIT © Steph Ango),
reading view by default, readable line length (~704px).

- **Committed (curated):** `docs/.obsidian/app.json`, `appearance.json`, `core-plugins.json`, `snippets/docs-reading.css`, `themes/Minimal/` (`theme.css`, `manifest.json`, `LICENSE`).
- **Gitignored (volatile/personal):** everything else under `docs/.obsidian/` - `workspace.json`, `workspace-mobile.json`, plugins, caches, and any themes/snippets you add locally.
- The docs remain plain Markdown: GitHub/VSCode rendering is unaffected, and agents (AGENTS.md) do not depend on Obsidian.
