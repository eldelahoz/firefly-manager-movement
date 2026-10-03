import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import { mockSettings, mockTags, mockCategories, mockBudgets } from '../mocks';
import { fixtures } from '../fixtures/fixtures';

async function openCounterpartyDropdown(page: Page, rowText: string) {
  await page.goto(`/results/${fixtures.imports[0].id}`);
  await page.getByRole('tab', { name: /Missing/ }).click();
  await page.getByRole('row').filter({ hasText: rowText }).getByRole('button', { name: 'Edit' }).click();
  await page.getByRole('combobox', { name: 'Counterparty account' }).click();
  return page.getByRole('listbox');
}

test.describe('Results > counterparty account types', () => {
  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
    await page.route('**/api/imports/1', async (route) => {
      await route.fulfill({
        json: {
          batch: fixtures.imports[0],
          movements: [],
          headers: [],
          detected_delimiter: ',',
        },
      });
    });
  });

  test('deposit offers revenue accounts only', async ({ page }) => {
    const listbox = await openCounterpartyDropdown(page, 'Salary');
    await expect(listbox.getByRole('option', { name: /Employer/ })).toBeVisible();
    await expect(listbox.getByRole('option', { name: /Checking/ })).toHaveCount(0);
    await expect(listbox.getByRole('option', { name: /Savings/ })).toHaveCount(0);
    await expect(listbox.getByRole('option', { name: /Supermarket/ })).toHaveCount(0);
  });

  test('withdrawal offers expense accounts only', async ({ page }) => {
    const listbox = await openCounterpartyDropdown(page, 'Book');
    await expect(listbox.getByRole('option', { name: /Supermarket/ })).toBeVisible();
    await expect(listbox.getByRole('option', { name: /Employer/ })).toHaveCount(0);
    await expect(listbox.getByRole('option', { name: /Checking/ })).toHaveCount(0);
    await expect(listbox.getByRole('option', { name: /Savings/ })).toHaveCount(0);
  });
});
