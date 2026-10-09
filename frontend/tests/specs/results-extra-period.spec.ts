import { test, expect } from '../fixtures/api.fixture';
import { mockSettings, mockTags, mockCategories, mockBudgets } from '../mocks';
import { fixtures } from '../fixtures/fixtures';

const detail = {
  batch: fixtures.imports[0],
  movements: [
    { id: 11, line_no: 2, date: '2026-08-01', amount_cents: -4200, description: 'Coffee', status: 'matched' },
    { id: 13, line_no: 4, date: '2026-08-05', amount_cents: 250000, description: 'Salary', status: 'missing' },
  ],
  headers: [],
  detected_delimiter: ',',
};
const results = {
  ...fixtures.results,
  extra: [
    { id: 8, ff3_id: 43, date: '2026-08-03', amount_cents: -1500, description: 'Duplicate A', is_duplicate: true, created_at: '2026-08-03T00:00:00Z' },
    { id: 9, ff3_id: 44, date: '2026-07-25', amount_cents: -1500, description: 'Old fee', is_duplicate: false, created_at: '2026-07-25T00:00:00Z' },
  ],
};

test.describe('Results > extras outside statement period', () => {
  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
    await page.route('**/api/imports/1', async (route) => route.fulfill({ json: detail }));
    await page.route('**/api/imports/1/results', async (route) => route.fulfill({ json: results }));
  });

  test('X1 out-of-period extra is badged and excluded from the count', async ({ page }) => {
    await page.goto('/results/1');
    await expect(page.getByRole('tab', { name: /Extra \(1\)/ })).toBeVisible();
    await page.getByRole('tab', { name: /Extra/ }).click();
    const oldFee = page.getByRole('row').filter({ hasText: 'Old fee' });
    await expect(oldFee.getByText('Outside period')).toBeVisible();
    await expect(oldFee.getByRole('button', { name: 'Delete duplicate' })).toHaveCount(0);
    const dup = page.getByRole('row').filter({ hasText: 'Duplicate A' });
    await expect(dup.getByText('Outside period')).toHaveCount(0);
  });
});
