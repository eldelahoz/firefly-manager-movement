import { test, expect } from '@playwright/test';
import { mockAccounts, mockHealth } from '../mocks';
import { mockImports } from '../mocks/imports.mock';

test.describe('Dashboard > HealthCard', () => {
  test.beforeEach(async ({ page }) => {
    await mockAccounts(page);
    await mockImports(page);
  });

  test('H1 ok without FF3 probe is not reported as connected', async ({ page }) => {
    await mockHealth(page);
    await page.goto('/');
    // Header chip + dashboard card both say unknown.
    await expect(page.getByText('Health unknown')).toHaveCount(2);
    await expect(page.getByText('Firefly III connected')).toHaveCount(0);
  });

  test('H2 reachable FF3: card hidden, only the sidebar chip says connected', async ({ page }) => {
    await mockHealth(page, { ff3: { reachable: true, version: '6.6.6' } });
    await page.goto('/');
    await expect(page.getByText('Firefly III connected')).toHaveCount(1);
    await expect(page.getByText('FF3 v6.6.6')).toHaveCount(0);
  });
});
