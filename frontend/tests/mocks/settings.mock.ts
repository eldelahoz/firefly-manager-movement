import type { Page, Route } from '@playwright/test';
import { fixtures } from '../fixtures/fixtures';

const { settings, tags, categories, budgets } = fixtures;

interface SettingsMockOptions {
  /** Override initial settings response */
  settingsOverrides?: Partial<typeof settings>;
}

export async function mockSettings(
  page: Page,
  opts: SettingsMockOptions = {},
): Promise<void> {
  const response = { ...settings, ...opts.settingsOverrides };

  await page.route('**/api/settings', async (route: Route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({ json: response });
    } else if (route.request().method() === 'PUT') {
      const body = route.request().postDataJSON();
      await route.fulfill({ json: { ...response, ...body } });
    } else {
      await route.fulfill({ status: 405, json: { error: 'Method Not Allowed' } });
    }
  });
}

export async function mockVerifySettings(page: Page): Promise<void> {
  await page.route('**/api/settings/verify', async (route: Route) => {
    await route.fulfill({ json: { version: '6.1.36' } });
  });
}

export async function mockTags(page: Page): Promise<void> {
  await page.route('**/api/firefly/tags', async (route: Route) => {
    await route.fulfill({ json: tags });
  });
}

export async function mockCategories(page: Page): Promise<void> {
  await page.route('**/api/firefly/categories', async (route: Route) => {
    await route.fulfill({ json: categories });
  });
}

export async function mockBudgets(page: Page): Promise<void> {
  await page.route('**/api/firefly/budgets', async (route: Route) => {
    await route.fulfill({ json: budgets });
  });
}
