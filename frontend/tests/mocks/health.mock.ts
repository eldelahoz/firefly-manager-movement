import type { Page, Route } from '@playwright/test';

interface HealthMockOptions {
  status?: 'ok' | 'degraded';
}

export async function mockHealth(
  page: Page,
  opts: HealthMockOptions = {},
): Promise<void> {
  const { status = 'ok' } = opts;

  await page.route('**/health', async (route: Route) => {
    await route.fulfill({ json: { status } });
  });
}
