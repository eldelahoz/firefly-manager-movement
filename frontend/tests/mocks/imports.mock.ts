import type { Page, Route } from '@playwright/test';
import { fixtures } from '../fixtures/fixtures';

const { imports } = fixtures;

export async function mockImports(page: Page): Promise<void> {
  await page.route('**/api/imports', async (route: Route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({ json: imports });
    } else {
      await route.fulfill({ status: 405, json: { error: 'Method Not Allowed' } });
    }
  });
}
