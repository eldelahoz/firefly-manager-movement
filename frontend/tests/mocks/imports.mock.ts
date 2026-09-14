import type { Page, Route } from '@playwright/test';
import { fixtures } from '../fixtures/fixtures';

const { imports } = fixtures;

export type MockImportsOptions = {
  imports?: typeof fixtures.imports;
  deleteDelayMs?: number;
};

export async function mockImports(page: Page, opts: MockImportsOptions = {}): Promise<void> {
  const data = opts.imports ?? imports;
  // Results has two path segments after /imports — neither `**/api/imports/*`
  // nor `**/api/imports*` matches it, and an unmocked 502 fires a sonner error
  // toast (queryCache.onError) whose <ol> breaks getByRole('list') strictness.
  await page.route('**/api/imports/*/results*', async (route: Route) => {
    await route.fulfill({ json: fixtures.results });
  });
  // Handle DELETE separately with more specific pattern
  await page.route('**/api/imports/*', async (route: Route) => {
    const url = route.request().url();
    const method = route.request().method();
    if (method === 'DELETE') {
      const deleteMatch = url.match(/\/api\/imports\/(\d+)(?:\?.*)?$/);
      if (deleteMatch) {
        const id = Number(deleteMatch[1]);
        const found = data.find((b) => b.id === id);
        if (!found) {
          await route.fulfill({ status: 404, json: { error: 'Not found' } });
          return;
        }
        if (found.status !== 'parsed') {
          await route.fulfill({ status: 400, json: { error: 'Only parsed can be deleted' } });
          return;
        }
        if (opts.deleteDelayMs) await new Promise((r) => setTimeout(r, opts.deleteDelayMs));
        await route.fulfill({ json: { id, status: 'deleted' } });
        return;
      }
    }
    await route.fallback();
  });
  await page.route('**/api/imports*', async (route: Route) => {
    const url = route.request().url();
    const method = route.request().method();
    if (url.includes('/results')) {
      await route.fulfill({ json: fixtures.results });
      return;
    }
    if (method === 'GET') {
      await route.fulfill({ json: data });
    } else {
      await route.fallback();
    }
  });
}
