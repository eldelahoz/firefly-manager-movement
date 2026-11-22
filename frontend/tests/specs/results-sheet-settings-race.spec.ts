import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import { mockSettings, mockTags, mockCategories, mockBudgets } from '../mocks';
import { fixtures } from '../fixtures/fixtures';

// F10 (review #12): the edit sheet opens before GET /api/settings answers, so the default tag is not known yet.
const detail = { batch: fixtures.imports[0], movements: [], headers: [], detected_delimiter: ',' };
const BOOK = {
  id: 12,
  line_no: 3,
  date: '2026-08-02',
  amount_cents: -9900,
  description: 'Book',
  status: 'missing',
};
const results = {
  reconciliation_id: 1,
  matched: [],
  missing: [BOOK],
  ignored: [],
  extra: [],
};

type ApplyBody = { movements: { movement_id: number; tags?: string[] }[] };

const sheet = (page: Page) => page.getByRole('dialog', { name: 'Edit movement' });

test.describe('Results > edit sheet opened before settings load', () => {
  let release: () => void;
  let applied: ApplyBody[];

  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
    // Registered last, so it wins: GET /settings waits for the gate, other methods fall back to the mock.
    const gate = new Promise<void>((resolve) => (release = resolve));
    await page.route('**/api/settings', async (route) => {
      if (route.request().method() !== 'GET') return route.fallback();
      await gate;
      await route.fulfill({ json: fixtures.settings });
    });
    await page.route('**/api/imports/1', (r) => r.fulfill({ json: detail }));
    await page.route('**/api/imports/1/results', (r) => r.fulfill({ json: results }));
    applied = [];
    await page.route('**/api/imports/1/apply', async (r) => {
      applied.push(r.request().postDataJSON() as ApplyBody);
      await r.fulfill({
        json: {
          reports: [{ movement_id: BOOK.id, outcome: 'created' }],
          created_count: 1,
          already_created_count: 0,
          failed_count: 0,
        },
      });
    });
    await page.goto('/results/1');
    await page
      .getByRole('row')
      .filter({ hasText: 'Book' })
      .getByRole('button', { name: 'Edit' })
      .click();
    await expect(sheet(page)).toBeVisible();
  });

  async function saveAndCreate(page: Page) {
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    await expect(sheet(page)).toHaveCount(0);
    await page.getByRole('row').filter({ hasText: 'Book' }).getByRole('checkbox').check();
    await page.getByRole('button', { name: 'Create in Firefly III (1)' }).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
    await expect.poll(() => applied.length).toBe(1);
    return applied[0].movements[0];
  }

  test('R3 default tag arriving after the sheet opened still applies when tags are untouched', async ({
    page,
  }) => {
    release();
    await expect(
      sheet(page).getByRole('button', { name: 'ffm-import', pressed: true }),
    ).toBeVisible();
    const sent = await saveAndCreate(page);
    expect(sent.tags).toEqual(['ffm-import']);
  });

  test('R4 the payload carries the empty tag selection of the user, not the default', async ({
    page,
  }) => {
    const pick = sheet(page).getByRole('button', { name: 'ffm-import' });
    await pick.click();
    await expect(pick).toHaveAttribute('aria-pressed', 'true');
    await pick.click();
    await expect(pick).toHaveAttribute('aria-pressed', 'false');
    release();
    await expect(pick).toHaveAttribute('aria-pressed', 'false');
    const sent = await saveAndCreate(page);
    expect(sent.tags).toEqual([]);
  });
});
