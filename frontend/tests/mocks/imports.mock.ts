import type { Page, Route } from '@playwright/test';
import { fixtures } from '../fixtures/fixtures';

const { imports } = fixtures;

export async function mockImports(page: Page): Promise<void> {
  await page.route('**/api/imports*', async (route: Route) => {
    const url = route.request().url();
    if (url.includes('/results')) {
      await route.fulfill({ json: fixtures.results });
      return;
    }
    if (route.request().method() === 'GET') {
      await route.fulfill({ json: imports });
    } else {
      await route.fulfill({ status: 405, json: { error: 'Method Not Allowed' } });
    }
  });
  // Mock for GET /api/imports/{id}/results used by KpiCards
  await page.route('**/api/imports/*/results*', async (route: Route) => {
    await route.fulfill({ json: fixtures.results });
  });
}
