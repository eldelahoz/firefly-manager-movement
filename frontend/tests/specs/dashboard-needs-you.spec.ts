import type { Page } from '@playwright/test';
import { test, expect } from '@playwright/test';
import { mockAccounts, mockHealth } from '../mocks';
import { mockImports } from '../mocks/imports.mock';
import { fixtures } from '../fixtures/fixtures';

// M5: dashboard "Needs you", Recent batches, health card visibility, first-run empty state.
const base = fixtures.imports[0];
const RECONCILED = {
  ...base,
  id: 1,
  filename: 'bank.csv',
  status: 'reconciled',
  movements_count: 5,
};
const PARSED = {
  ...base,
  id: 2,
  filename: 'half.csv',
  status: 'parsed',
  movements_count: 3,
  created_at: '2026-08-19T10:00:00Z',
};

async function setup(page: Page, imports: (typeof RECONCILED)[], health = {}, results?: unknown) {
  await mockHealth(page, health);
  await mockAccounts(page);
  await mockImports(page, { imports });
  // Registered after mockImports: the last matching route wins.
  if (results) await page.route('**/api/imports/1/results', (r) => r.fulfill({ json: results }));
  // The card also reads the statement period from the batch detail (extras outside it are not counted).
  await page.route('**/api/imports/1', async (route) => {
    if (route.request().method() !== 'GET') return route.fallback();
    await route.fulfill({
      json: {
        batch: RECONCILED,
        movements: [
          { line_no: 2, date: '2026-08-01', amount_cents: -4200, description: 'Coffee' },
          { line_no: 4, date: '2026-08-05', amount_cents: 250000, description: 'Salary' },
        ],
        headers: ['date', 'amount', 'desc'],
        detected_delimiter: ',',
      },
    });
  });
  await page.route('**/api/imports/2', async (route) => {
    if (route.request().method() !== 'GET') return route.fallback();
    await route.fulfill({
      json: {
        batch: PARSED,
        movements: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
        headers: ['date', 'amount', 'desc'],
        detected_delimiter: ',',
      },
    });
  });
  await page.goto('/');
}

const needsYou = (page: Page) => page.locator('section[aria-label="Needs you"]');
const recent = (page: Page) => page.locator('section[aria-label="Recent batches"]');

test.describe('Dashboard > Needs you', () => {
  test('D1 latest reconciled batch: missing count, Review link, legend', async ({ page }) => {
    await setup(page, [RECONCILED, PARSED]);
    const card = needsYou(page);
    await expect(card.getByRole('heading', { name: 'Needs you' })).toBeVisible();
    await expect(
      card.getByRole('listitem').filter({ hasText: 'bank.csv · 2 to create' }),
    ).toBeVisible();
    await expect(card.getByRole('link', { name: 'Review bank.csv' })).toHaveAttribute(
      'href',
      '/results/1',
    );
    for (const [group, text] of [
      ['matched', '1 matched'],
      ['missing', '2 missing'],
      ['extra', '1 extra'],
      ['ignored', '1 ignored'],
    ]) {
      await expect(card.locator(`li[data-group="${group}"]`)).toHaveText(text);
    }
  });

  test('D1b extras dated outside the statement period are not counted (same as Results)', async ({
    page,
  }) => {
    const outside = { ...fixtures.results.extra[0], id: 9, ff3_id: 44, date: '2026-07-25' };
    await setup(
      page,
      [RECONCILED],
      {},
      {
        ...fixtures.results,
        extra: [...fixtures.results.extra, outside],
      },
    );
    await expect(needsYou(page).locator('li[data-group="extra"]')).toHaveText('1 extra');
  });

  test('D2 nothing missing reads All reconciled', async ({ page }) => {
    await setup(page, [RECONCILED], {}, { ...fixtures.results, missing: [] });
    await expect(needsYou(page).getByText('bank.csv · All reconciled')).toBeVisible();
  });

  test('D3 unfinished import offers Resume, which opens the wizard', async ({ page }) => {
    await setup(page, [RECONCILED, PARSED]);
    const card = needsYou(page);
    await expect(
      card.getByRole('listitem').filter({ hasText: 'half.csv · Import not finished' }),
    ).toBeVisible();
    await card.getByRole('button', { name: 'Resume half.csv' }).click();
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page.getByRole('button', { name: /Reconcile/ })).toBeVisible({ timeout: 5000 });
  });

  test('D4 nothing to do: the card says so; empty parsed batch is not resumable', async ({
    page,
  }) => {
    await setup(page, [{ ...PARSED, movements_count: 0 }]);
    await expect(needsYou(page).getByText('Nothing needs you right now.')).toBeVisible();
    await expect(needsYou(page).getByRole('button', { name: /Resume/ })).toHaveCount(0);
  });

  test('D5 the KPI grid is gone', async ({ page }) => {
    await setup(page, [RECONCILED]);
    await expect(page.getByText('Total batches')).toHaveCount(0);
  });

  test('D6 Recent batches: Review link for reconciled, Resume button for parsed', async ({
    page,
  }) => {
    await setup(page, [RECONCILED, PARSED]);
    const table = recent(page);
    await expect(table.getByRole('button', { name: 'Resume half.csv' })).toBeVisible();
    await table.getByRole('link', { name: 'Review bank.csv' }).click();
    await expect(page).toHaveURL(/\/results\/1$/);
  });

  test('D7 first run: one card with New import, no Needs you / Recent batches', async ({
    page,
  }) => {
    await setup(page, []);
    const main = page.getByRole('main');
    await expect(main.getByText('Import your first bank statement')).toBeVisible();
    await expect(needsYou(page)).toHaveCount(0);
    await expect(recent(page)).toHaveCount(0);
    await main.getByRole('button', { name: 'New import' }).last().click();
    await expect(page.getByRole('dialog')).toBeVisible();
  });

  test.describe('health card visibility', () => {
    test('D8 hidden when connected; the version is not shown', async ({ page }) => {
      await setup(page, [RECONCILED], { ff3: { reachable: true, version: '6.6.6' } });
      await expect(page.getByText('Firefly III connected')).toHaveCount(1); // sidebar chip only
      await expect(page.getByText('FF3 v6.6.6')).toHaveCount(0);
    });

    test('D9 shown as unknown when there is no probe', async ({ page }) => {
      await setup(page, [RECONCILED]);
      await expect(page.getByText('Health unknown')).toHaveCount(2);
    });

    test('D10 shown as an alert when the backend is degraded', async ({ page }) => {
      await setup(page, [RECONCILED], { status: 'degraded' });
      await expect(page.getByRole('main').getByRole('alert')).toBeVisible();
    });
  });

  test.describe('FF3 down: query-error toasts', () => {
    const toast = (page: Page) => page.locator('[data-sonner-toast][data-type="error"]');
    async function down(page: Page, status: number) {
      await mockHealth(page, { status: 'degraded' });
      await mockAccounts(page);
      await page.route('**/api/imports', (r) => r.fulfill({ status, json: { error: 'boom' } }));
      await page.goto('/');
      await expect(page.getByRole('main').getByRole('alert')).toBeVisible();
    }

    test('D12 a real server error (500) still toasts while FF3 is down', async ({ page }) => {
      await down(page, 500);
      await expect(toast(page)).toBeVisible();
    });

    test('D13 a 502 while FF3 is down only shows the health alert, no duplicate toast', async ({
      page,
    }) => {
      await down(page, 502);
      await page.waitForTimeout(1500); // a toast would have appeared by now
      await expect(toast(page)).toHaveCount(0);
    });
  });

  test.describe('at 390px', () => {
    test.use({ viewport: { width: 390, height: 844 } });

    test('D11 no horizontal overflow; Needs you actions stay inside the viewport', async ({
      page,
    }) => {
      await setup(page, [RECONCILED, PARSED]);
      await expect(needsYou(page)).toBeVisible();
      const over = await page.evaluate(
        () => document.documentElement.scrollWidth - window.innerWidth,
      );
      expect(over).toBeLessThanOrEqual(0);
      for (const el of [
        needsYou(page).getByRole('link', { name: 'Review bank.csv' }),
        needsYou(page).getByRole('button', { name: 'Resume half.csv' }),
      ]) {
        const box = (await el.boundingBox())!;
        expect(box.x).toBeGreaterThanOrEqual(0);
        expect(box.x + box.width).toBeLessThanOrEqual(390);
      }
    });
  });

  for (const failing of ['detail', 'results'] as const) {
    test(`D14 failed batch ${failing} shows an inline error with Retry; the row stays`, async ({
      page,
    }) => {
      await mockHealth(page);
      await mockAccounts(page);
      await mockImports(page, { imports: [RECONCILED] });
      let fail = true;
      const detail = {
        batch: RECONCILED,
        movements: [{ line_no: 2, date: '2026-08-01', amount_cents: -4200, description: 'Coffee' }],
        headers: ['date', 'amount', 'desc'],
        detected_delimiter: ',',
      };
      await page.route('**/api/imports/1', async (route) => {
        if (route.request().method() !== 'GET') return route.fallback();
        if (fail && failing === 'detail') return route.fulfill({ status: 500, json: { error: 'x' } });
        await route.fulfill({ json: detail });
      });
      await page.route('**/api/imports/1/results', (route) =>
        fail && failing === 'results'
          ? route.fulfill({ status: 500, json: { error: 'x' } })
          : route.fulfill({ json: fixtures.results }),
      );
      await page.goto('/');
      const card = needsYou(page);
      // The error toast is an alert too, but outside the card.
      const alert = card.getByRole('alert');
      await expect(alert).toContainText('Something went wrong');
      await expect(card.getByText('bank.csv')).toBeVisible();
      await expect(card.getByRole('link', { name: 'Review bank.csv' })).toBeVisible();
      await expect(card.locator('li[data-group]')).toHaveCount(0);

      fail = false;
      await alert.getByRole('button', { name: 'Retry' }).click();
      await expect(card.locator('li[data-group="missing"]')).toHaveText('2 missing');
      await expect(alert).toHaveCount(0);
    });
  }

  test('D11 failed imports query shows an inline error with Retry that recovers', async ({
    page,
  }) => {
    await mockHealth(page);
    await mockAccounts(page);
    let fail = true;
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() !== 'GET') return route.fallback();
      if (fail) return route.fulfill({ status: 500, json: { error: 'boom' } });
      await route.fulfill({ json: [RECONCILED] });
    });
    await page.route('**/api/imports/1', (r) =>
      r.fulfill({
        json: { batch: RECONCILED, movements: [], headers: [], detected_delimiter: ',' },
      }),
    );
    await page.route('**/api/imports/1/results', (r) => r.fulfill({ json: fixtures.results }));
    await page.goto('/');
    // The global error toast also has an alert-like role: pick the alert that holds Retry.
    const alert = page
      .getByRole('alert')
      .filter({ has: page.getByRole('button', { name: 'Retry' }) });
    await expect(alert).toBeVisible();
    await expect(needsYou(page)).toHaveCount(0);
    fail = false;
    await alert.getByRole('button', { name: 'Retry' }).click();
    await expect(needsYou(page)).toBeVisible();
    await expect(alert).toHaveCount(0);
  });
});
