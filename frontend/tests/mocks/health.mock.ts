import type { Page, Route } from '@playwright/test';

interface HealthMockOptions {
  status?: 'ok' | 'degraded';
  ff3?: { reachable: boolean; version?: string };
}

export async function mockHealth(
  page: Page,
  opts: HealthMockOptions = {},
): Promise<void> {
  const { status = 'ok', ff3 } = opts;

  await page.route('**/health*', async (route: Route) => {
    await route.fulfill({ json: { status, ff3 } });
  });
}
