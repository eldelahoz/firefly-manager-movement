# firefly-manager-movement — Frontend

React 19 + TypeScript + Vite (proxy `/api` → `http://localhost:3000`, `/health` → `:3000`).

## Scripts

| Command | Purpose |
|---------|---------|
| `pnpm dev` | Vite dev server `http://localhost:5173` (proxies `/api` to `:3000`) |
| `pnpm lint` | `eslint . && prettier --check src` |
| `pnpm build` | `tsc -b && vite build` |
| `pnpm e2e` | **Único runner de tests** — `playwright test` (`tests/specs/**`, `baseURL http://localhost:5173`, `webServer pnpm dev`) |
| `pnpm e2e --ui` | Playwright UI mode |
| `npx playwright show-report` | HTML report |
| `npx playwright test --headed` | headed run rápido |

Tests: `tests/specs/dashboard-browse.spec.ts` (6 casos R1.1–R1.6, `page.route` mocks en `tests/mocks/**`, fixtures `tests/fixtures/fixtures.ts` + `tests/fixtures/api.fixture.ts`). Gaps Playwright pendientes en `docs/superpowers/plans/2026-09-09-purge-vitest-only-playwright.md`.

## Stack

- React 19, React Router, TanStack Query + Virtual, React Hook Form + Zod
- Tailwind v4 + radix-ui, lucide
- Playwright 1.63 (solo runner; Vitest/MSW/@testing-library eliminados)
