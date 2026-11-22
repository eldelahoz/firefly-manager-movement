import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import { mockSettings, mockTags, mockCategories, mockBudgets } from '../mocks';
import { fixtures } from '../fixtures/fixtures';

// M1: optimistic ignore + Undo (local ignore only; create/delete keep their confirmation, no Undo).
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
const DUP = {
  id: 8,
  ff3_id: 43,
  date: '2026-08-03',
  amount_cents: -1500,
  description: 'Duplicate A',
  is_duplicate: true,
  created_at: '2026-08-03T00:00:00Z',
};

type Opts = {
  ignoreStatus?: number;
  ignoreDelayMs?: number;
  failResultsAfterIgnore?: boolean;
  /** Book starts in Ignored instead of Missing. */
  bookIgnored?: boolean;
};

/** Stateful results mock: POST ignore moves a movement between missing/ignored like the backend. */
async function mockResults(page: Page, opts: Opts = {}) {
  const state = opts.bookIgnored
    ? { missing: [SALARY], ignored: [{ ...BOOK, status: 'ignored' }], extra: [DUP] }
    : { missing: [BOOK, SALARY], ignored: [] as (typeof BOOK)[], extra: [DUP] };
  const posts: { id: number; ignored: boolean }[] = [];
  const failedGets = { count: 0 };
  await page.route('**/api/imports/1', (r) => r.fulfill({ json: detail }));
  let ignoreSeen = false;
  await page.route('**/api/imports/1/results', (r) => {
    if (opts.failResultsAfterIgnore && ignoreSeen) {
      failedGets.count++;
      return r.fulfill({ status: 500, json: { error: 'boom' } });
    }
    return r.fulfill({ json: { ...fixtures.results, matched: [], ...state } });
  });
  await page.route('**/api/imports/1/movements/*/ignore', async (route) => {
    const id = Number(
      route
        .request()
        .url()
        .match(/movements\/(\d+)\/ignore/)![1],
    );
    const { ignored } = route.request().postDataJSON() as { ignored: boolean };
    posts.push({ id, ignored });
    ignoreSeen = true;
    if (opts.ignoreDelayMs) await new Promise((r) => setTimeout(r, opts.ignoreDelayMs));
    if (opts.ignoreStatus && opts.ignoreStatus >= 400) {
      await route.fulfill({ status: opts.ignoreStatus, json: { error: 'boom' } });
      return;
    }
    const [from, to] = ignored ? [state.missing, state.ignored] : [state.ignored, state.missing];
    const m = from.find((x) => x.id === id)!;
    from.splice(from.indexOf(m), 1);
    to.push({ ...m, status: ignored ? 'ignored' : 'missing' });
    await route.fulfill({ json: { id, status: ignored ? 'ignored' : 'missing' } });
  });
  return { state, posts, failedGets };
}

// Scoped to the open tab: the Ignored section below also lists the row once it moves.
const row = (page: Page, text: string) =>
  page.getByRole('tabpanel').getByRole('row').filter({ hasText: text });
// M2: the Ignored table only renders while its collapsible is open.
async function openIgnored(page: Page) {
  const details = page.locator('section[aria-label="Ignored"] details');
  if ((await details.getAttribute('open')) === null) await details.locator('summary').click();
  return page.getByRole('table', { name: 'Ignored' });
}
const undo = (page: Page) => page.getByRole('button', { name: 'Undo', exact: true });

test.describe('Results > optimistic ignore + Undo', () => {
  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
  });

  async function ignoreBook(page: Page) {
    await page.goto('/results/1');
    await page.getByRole('tab', { name: /Missing/ }).click();
    await row(page, 'Book').getByRole('button', { name: 'Ignore' }).click();
  }

  test('U1 row leaves immediately, before the server answers', async ({ page }) => {
    const { posts } = await mockResults(page, { ignoreDelayMs: 3000 });
    await ignoreBook(page);
    await expect(row(page, 'Book')).toHaveCount(0, { timeout: 1000 });
    await expect(row(page, 'Salary')).toBeVisible();
    await expect(undo(page)).toBeVisible();
    expect(posts).toEqual([{ id: 12, ignored: true }]);
  });

  test('U2 Undo restores the row and sends un-ignore', async ({ page }) => {
    const { posts } = await mockResults(page);
    await ignoreBook(page);
    await expect(row(page, 'Book')).toHaveCount(0);
    await undo(page).click();
    await expect(row(page, 'Book')).toBeVisible();
    expect(posts).toEqual([
      { id: 12, ignored: true },
      { id: 12, ignored: false },
    ]);
  });

  test('U3 Undo lasts the 8 s window, then the toast is gone', async ({ page }) => {
    await page.clock.install();
    await mockResults(page);
    await ignoreBook(page);
    await expect(undo(page)).toBeVisible();
    await page.clock.runFor(6000);
    await expect(undo(page)).toBeVisible();
    await page.clock.runFor(2500);
    await expect(undo(page)).toHaveCount(0);
    await expect(row(page, 'Book')).toHaveCount(0);
  });

  test('U8 un-ignore while the ignore POST is pending: requests stay ordered, Book ends in Missing', async ({
    page,
  }) => {
    const { state, posts } = await mockResults(page, { ignoreDelayMs: 1000 });
    await ignoreBook(page);
    await (
      await openIgnored(page)
    )
      .getByRole('row')
      .filter({ hasText: 'Book' })
      .getByRole('button', { name: 'Un-ignore' })
      .click();
    await expect(row(page, 'Book')).toBeVisible();
    await expect.poll(() => posts.length, { timeout: 8000 }).toBe(2);
    expect(posts).toEqual([
      { id: 12, ignored: true },
      { id: 12, ignored: false },
    ]);
    // both responses and the refetch settled: server state and UI agree
    await expect.poll(() => state.missing.map((m) => m.id)).toContain(12);
    await expect(row(page, 'Book')).toBeVisible();
  });

  test('U4 server error rolls the row back with an error toast and no Undo', async ({ page }) => {
    await mockResults(page, { ignoreStatus: 500 });
    await ignoreBook(page);
    await expect(row(page, 'Book')).toBeVisible();
    await expect(page.locator('[data-sonner-toast][data-type="error"]')).toBeVisible();
    await expect(undo(page)).toHaveCount(0);
  });

  test('U5 reduced motion: row still leaves for good and Undo still works', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await mockResults(page);
    await ignoreBook(page);
    await expect(row(page, 'Book')).toHaveCount(0); // removed from DOM, not just faded
    await undo(page).click();
    await expect(row(page, 'Book')).toBeVisible();
  });

  test('U6 create shows no Undo', async ({ page }) => {
    await mockResults(page);
    await page.route('**/api/imports/1/apply', (r) =>
      r.fulfill({
        json: {
          reports: [{ movement_id: 12, outcome: 'created', ff3_id: 99 }],
          created_count: 1,
          already_created_count: 0,
          failed_count: 0,
        },
      }),
    );
    await page.goto('/results/1');
    await page.getByRole('tab', { name: /Missing/ }).click();
    await row(page, 'Book').getByRole('checkbox').click();
    await page.getByRole('button', { name: /Create in Firefly III \(1\)/ }).click();
    await expect(undo(page)).toHaveCount(0);
    await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
    await expect(page.locator('[data-sonner-toast]').first()).toBeVisible();
    await expect(undo(page)).toHaveCount(0);
  });

  test('U7 delete duplicate shows no Undo', async ({ page }) => {
    await mockResults(page);
    await page.route('**/api/firefly/transactions/43', (r) =>
      r.fulfill({ json: { ff3_id: 43, status: 'deleted' } }),
    );
    await page.goto('/results/1');
    await page.getByRole('tab', { name: /Extra/ }).click();
    await row(page, 'Duplicate A').getByRole('button', { name: 'Delete duplicate' }).click();
    const dialog = page.getByRole('dialog');
    await dialog.getByRole('button', { name: 'Delete', exact: true }).click();
    await dialog.getByRole('button', { name: 'Confirm', exact: true }).click();
    await expect(page.getByText('Deleted (audit saved)')).toBeVisible();
    await expect(undo(page)).toHaveCount(0);
  });

  test('U9 focus moves to the same action button of the next row, then to the active tab when Missing is empty', async ({
    page,
  }) => {
    await mockResults(page);
    await ignoreBook(page);
    await expect(row(page, 'Salary').getByRole('button', { name: 'Ignore' })).toBeFocused();
    await row(page, 'Salary').getByRole('button', { name: 'Ignore' }).click();
    await expect(row(page, 'Salary')).toHaveCount(0);
    // Missing is now empty (the page switches tab): focus falls back to whichever tab is active.
    await expect(page.locator('[role="tab"][data-state="active"]')).toBeFocused();
  });

  test('U10 Undo from the toast keeps focus inside the Missing table (not on body)', async ({
    page,
  }) => {
    await mockResults(page);
    await ignoreBook(page);
    await undo(page).click();
    await expect(row(page, 'Book')).toBeVisible();
    // Sonner hands focus back to the previously focused row button after our handler runs.
    await expect(
      page.locator('#results-missing-table :focus, #results-missing-table:focus'),
    ).toHaveCount(1);
  });

  test('U11 a failed results refetch keeps the optimistic state', async ({ page }) => {
    await mockResults(page, { failResultsAfterIgnore: true });
    await ignoreBook(page);
    await expect(undo(page)).toBeVisible();
    await page.waitForTimeout(1000); // let the failed refetch land
    await expect(row(page, 'Book')).toHaveCount(0);
    await expect(
      (await openIgnored(page)).getByRole('row').filter({ hasText: 'Book' }),
    ).toBeVisible();
  });

  test('U13 Create sends a row un-ignored while the results refetch keeps failing', async ({
    page,
  }) => {
    const { failedGets } = await mockResults(page, {
      bookIgnored: true,
      failResultsAfterIgnore: true,
    });
    const applied: { movements: { movement_id: number }[] }[] = [];
    await page.route('**/api/imports/1/apply', async (r) => {
      applied.push(r.request().postDataJSON());
      await r.fulfill({
        json: {
          reports: [{ movement_id: 12, outcome: 'created', ff3_id: 99 }],
          created_count: 1,
          already_created_count: 0,
          failed_count: 0,
        },
      });
    });
    await page.goto('/results/1');
    await page.getByRole('tab', { name: /Missing/ }).click();
    await (
      await openIgnored(page)
    )
      .getByRole('row')
      .filter({ hasText: 'Book' })
      .getByRole('button', { name: 'Un-ignore' })
      .click();
    await expect(row(page, 'Book')).toBeVisible();
    await expect.poll(() => failedGets.count, { message: 'the refetch failed' }).toBeGreaterThan(0);
    await row(page, 'Book').getByRole('checkbox').check();
    await page.getByRole('button', { name: /Create in Firefly III \(1\)/ }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toContainText('Book');
    await dialog.getByRole('button', { name: 'Create', exact: true }).click();
    await expect.poll(() => applied.length).toBe(1);
    expect(applied[0].movements.map((m) => m.movement_id)).toEqual([12]);
  });

  test('U12 ignoring every missing row keeps the Missing tab selected and the Ignored section', async ({
    page,
  }) => {
    await mockResults(page);
    await ignoreBook(page);
    await row(page, 'Salary').getByRole('button', { name: 'Ignore' }).click();
    await expect(row(page, 'Salary')).toHaveCount(0);
    await expect(page.getByRole('tab', { name: /^Missing/ })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(page.locator('section[aria-label="Ignored"] summary')).toHaveText('Ignored (2)');
  });
});
