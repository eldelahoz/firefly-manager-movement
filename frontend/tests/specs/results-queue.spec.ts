import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import { mockSettings, mockTags, mockCategories, mockBudgets } from '../mocks';
import { fixtures } from '../fixtures/fixtures';

// M2: results review queue (summary, tab counts, bulk bar, outcome kept, Ignored collapsible, skeleton, 390).
const detail = { batch: fixtures.imports[0], movements: [], headers: [], detected_delimiter: ',' };
const BOOK = {
  id: 12,
  line_no: 3,
  date: '2026-08-02',
  amount_cents: -9900,
  description: 'Book',
  status: 'missing',
};
const SALARY = {
  id: 13,
  line_no: 4,
  date: '2026-08-05',
  amount_cents: 250000,
  description: 'Salary',
  status: 'missing',
};
const COFFEE = {
  movement: {
    id: 11,
    line_no: 2,
    date: '2026-08-01',
    amount_cents: -4200,
    description: 'Coffee',
    status: 'matched',
  },
  ff3: {
    id: 7,
    ff3_id: 42,
    date: '2026-08-03',
    amount_cents: -4200,
    description: 'Coffee',
    is_duplicate: false,
    created_at: '2026-08-03T00:00:00Z',
  },
};
const TRANSFER = {
  id: 14,
  line_no: 5,
  date: '2026-08-02',
  amount_cents: 1500,
  description: 'Internal transfer',
  status: 'ignored',
};
const DUP = {
  id: 8,
  ff3_id: 43,
  date: '2026-08-03',
  amount_cents: -1500,
  description: 'Duplicate A',
  is_duplicate: true,
  created_at: '2026-08-03T00:00:00Z',
};
const results = {
  reconciliation_id: 1,
  matched: [COFFEE],
  missing: [BOOK, SALARY],
  ignored: [TRANSFER],
  extra: [DUP],
};

async function mockResults(page: Page, body: unknown = results, delayMs = 0) {
  await page.route('**/api/imports/1', (r) => r.fulfill({ json: detail }));
  await page.route('**/api/imports/1/results', async (r) => {
    if (delayMs) await new Promise((res) => setTimeout(res, delayMs));
    await r.fulfill({ json: body });
  });
}

const summary = (page: Page) => page.getByRole('region', { name: 'Reconciliation summary' });
const ignoredSection = (page: Page) => page.locator('section[aria-label="Ignored"]');

test.describe('Results > review queue', () => {
  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
  });

  test('Q1 summary shows the headline and per-group counts', async ({ page }) => {
    await mockResults(page);
    await page.goto('/results/1');
    await expect(summary(page).getByText('2 to create')).toBeVisible();
    for (const [group, text] of [
      ['matched', '1 matched'],
      ['missing', '2 missing'],
      ['extra', '1 extra'],
      ['ignored', '1 ignored'],
    ]) {
      await expect(summary(page).locator(`[data-group="${group}"]`)).toHaveText(text);
    }
    await expect(summary(page).locator('[data-group="failed"]')).toHaveCount(0);
  });

  test('Q2 tab counts, Missing is the default tab when there is work', async ({ page }) => {
    await mockResults(page);
    await page.goto('/results/1');
    for (const name of ['Matched (1)', 'Missing (2)', 'Extra (1)']) {
      await expect(page.getByRole('tab', { name })).toBeVisible();
    }
    await expect(page.getByRole('tab', { name: 'Missing (2)' })).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('Q3 nothing missing: "All reconciled" and Matched is the default tab', async ({ page }) => {
    await mockResults(page, { ...results, missing: [] });
    await page.goto('/results/1');
    await expect(summary(page).getByText('All reconciled')).toBeVisible();
    await expect(page.getByRole('tab', { name: /Matched/ })).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });

  test('Q4 bulk bar exists only while rows are selected', async ({ page }) => {
    await mockResults(page);
    await page.goto('/results/1');
    const bar = page.getByRole('toolbar', { name: 'Selection' });
    await expect(bar).toHaveCount(0);
    await expect(page.getByRole('button', { name: /Create in Firefly III/ })).toHaveCount(0);
    await page.getByRole('row').filter({ hasText: 'Book' }).getByRole('checkbox').check();
    await expect(bar.getByText('1 selected')).toBeVisible();
    await expect(bar.getByRole('button', { name: 'Personalize (1)' })).toBeVisible();
    await expect(bar.getByRole('button', { name: 'Create in Firefly III (1)' })).toBeVisible();
    await page.getByRole('checkbox', { name: 'Select all' }).check();
    await expect(bar.getByText('2 selected')).toBeVisible();
    await bar.getByRole('button', { name: 'Clear selection' }).click();
    await expect(bar).toHaveCount(0);
  });

  test('Q5 apply keeps each row outcome and counts failures in the summary', async ({ page }) => {
    await mockResults(page);
    await page.route('**/api/imports/1/apply', (r) =>
      r.fulfill({
        json: {
          reports: [
            { movement_id: 12, outcome: 'created', ff3_id: 99 },
            { movement_id: 13, outcome: 'failed', reason: 'Boom from FF3' },
          ],
          created_count: 1,
          already_created_count: 0,
          failed_count: 1,
        },
      }),
    );
    await page.goto('/results/1');
    await page.getByRole('checkbox', { name: 'Select all' }).check();
    await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
    const book = page.getByRole('row').filter({ hasText: 'Book' });
    const salary = page.getByRole('row').filter({ hasText: 'Salary' });
    await expect(book.getByText('Created', { exact: true })).toBeVisible();
    await expect(salary.getByText('Failed', { exact: true })).toBeVisible();
    await expect(salary.getByText('Boom from FF3')).toBeVisible();
    await expect(summary(page).locator('[data-group="failed"]')).toHaveText('1 failed');
  });

  test('Q6 Ignored is a collapsible, closed by default, remembered, Missing tab only', async ({
    page,
  }) => {
    await mockResults(page);
    await page.goto('/results/1');
    const summaryEl = ignoredSection(page).locator('summary');
    await expect(summaryEl).toHaveText('Ignored (1)');
    await expect(page.getByRole('table', { name: 'Ignored' })).toHaveCount(0);
    await summaryEl.click();
    await expect(
      page.getByRole('table', { name: 'Ignored' }).getByText('Internal transfer'),
    ).toBeVisible();
    await page.reload();
    await expect(
      page.getByRole('table', { name: 'Ignored' }).getByText('Internal transfer'),
    ).toBeVisible();
    await page.getByRole('tab', { name: /Extra/ }).click();
    await expect(ignoredSection(page)).toHaveCount(0);
  });

  test('Q7 skeleton while results load', async ({ page }) => {
    await mockResults(page, results, 1500);
    await page.goto('/results/1');
    const skeleton = page.getByRole('status', { name: 'Loading results' });
    await expect(skeleton).toBeVisible();
    await expect(skeleton).toHaveAttribute('aria-busy', 'true');
    await expect(page.getByRole('tab', { name: 'Missing (2)' })).toBeVisible();
    await expect(skeleton).toHaveCount(0);
  });

  test('Q8 matched row flags a Firefly III date that differs from the statement', async ({
    page,
  }) => {
    await mockResults(page);
    await page.goto('/results/1');
    await page.getByRole('tab', { name: /Matched/ }).click();
    await expect(
      page
        .getByRole('row')
        .filter({ hasText: 'Coffee' })
        .getByText(/Firefly III.*\+2 d/),
    ).toBeVisible();
  });

  test.describe('at 390px', () => {
    test.use({ viewport: { width: 390, height: 844 } });

    test('Q9 Ignore and select-all stay reachable, no horizontal overflow', async ({ page }) => {
      await mockResults(page);
      await page.goto('/results/1');
      const ignore = page
        .getByRole('row')
        .filter({ hasText: 'Book' })
        .getByRole('button', { name: 'Ignore' });
      const selectAll = page.getByRole('checkbox', { name: 'Select all' });
      for (const el of [ignore, selectAll]) {
        await expect(el).toBeVisible();
        const box = await el.boundingBox();
        expect(box && box.x >= 0 && box.x + box.width <= 390, 'inside viewport width').toBe(true);
      }
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth - window.innerWidth,
      );
      expect(overflow).toBeLessThanOrEqual(0);
    });
  });
});
