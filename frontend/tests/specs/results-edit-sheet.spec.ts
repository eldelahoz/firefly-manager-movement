import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import { mockSettings, mockTags, mockCategories, mockBudgets } from '../mocks';
import { fixtures } from '../fixtures/fixtures';

// M3: edit sheet (single + bulk) replacing EditMovementDialog / BulkPersonalizeDialog.
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
const results = {
  reconciliation_id: 1,
  matched: [],
  missing: [BOOK, SALARY],
  ignored: [],
  extra: [],
};

type ApplyBody = { movements: Record<string, unknown>[] };

type Outcome = 'created' | 'failed';

/** `outcomes` maps a movement id to its apply outcome; unlisted ids produce no report. */
async function setup(
  page: Page,
  missing = results.missing,
  outcomes: Record<number, Outcome> = {},
) {
  await page.route('**/api/imports/1', (r) => r.fulfill({ json: detail }));
  await page.route('**/api/imports/1/results', (r) => r.fulfill({ json: { ...results, missing } }));
  const applied: ApplyBody[] = [];
  await page.route('**/api/imports/1/apply', async (r) => {
    const body = r.request().postDataJSON() as ApplyBody;
    applied.push(body);
    const reports = body.movements.flatMap((m) => {
      const outcome = outcomes[m.movement_id as number];
      return outcome
        ? [{ movement_id: m.movement_id, outcome, ...(outcome === 'failed' && { reason: 'boom' }) }]
        : [];
    });
    const count = (o: Outcome) => reports.filter((x) => x.outcome === o).length;
    await r.fulfill({
      json: {
        reports,
        created_count: count('created'),
        already_created_count: 0,
        failed_count: count('failed'),
      },
    });
  });
  await page.goto('/results/1');
  return applied;
}

/** The sheet slides in: wait until its box stops moving before measuring it. */
async function settled(el: import('@playwright/test').Locator) {
  let prev = '';
  await expect
    .poll(async () => {
      const cur = JSON.stringify(await el.boundingBox());
      const same = cur === prev;
      prev = cur;
      return same;
    })
    .toBe(true);
}

const sheet = (page: Page) => page.getByRole('dialog', { name: 'Edit movement' });
const guard = (page: Page) => page.getByRole('dialog', { name: 'Discard changes?' });
const editBtn = (page: Page, text: string) =>
  page.getByRole('row').filter({ hasText: text }).getByRole('button', { name: 'Edit' });
const openBook = async (page: Page) => {
  await editBtn(page, 'Book').click();
  await expect(sheet(page)).toBeVisible();
};
const makeDirty = (page: Page) => sheet(page).getByRole('textbox', { name: 'Notes' }).fill('n1');

async function createSelected(page: Page, rowText: string, n = 1) {
  if (n === 1)
    await page.getByRole('row').filter({ hasText: rowText }).getByRole('checkbox').check();
  await page.getByRole('button', { name: `Create in Firefly III (${n})` }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
}

test.describe('Results > edit sheet', () => {
  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
  });

  test('E1 opens prefilled with the statement data and position', async ({ page }) => {
    await setup(page);
    await openBook(page);
    await expect(sheet(page)).toContainText('Movement 1 of 2');
    await expect(sheet(page)).toContainText('Line 3');
    await expect(sheet(page).getByRole('textbox', { name: 'Description' })).toHaveValue('Book');
    await expect(sheet(page).getByRole('button', { name: 'Previous movement' })).toBeDisabled();
    await expect(sheet(page).getByRole('button', { name: 'Next movement' })).toBeEnabled();
    // RF-08: the tags picker is pre-filled with the default creation tag
    await expect(
      sheet(page).getByRole('button', { name: 'ffm-import', pressed: true }),
    ).toBeVisible();
    await expect(
      sheet(page).getByRole('button', { name: 'vacation', pressed: false }),
    ).toBeVisible();
  });

  test('E2Save keeps the fields locally; they ride on the next Create', async ({ page }) => {
    const applied = await setup(page);
    await openBook(page);
    await sheet(page).getByRole('textbox', { name: 'Description' }).fill('Books');
    await sheet(page).getByRole('textbox', { name: 'Notes' }).fill('n1');
    await sheet(page).getByRole('combobox', { name: 'Category' }).click();
    await page.getByRole('option', { name: 'Groceries' }).click();
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    await expect(sheet(page)).toHaveCount(0);
    expect(applied, 'Save must not call Firefly').toHaveLength(0);
    await createSelected(page, 'Book');
    await expect.poll(() => applied.length).toBe(1);
    expect(applied[0].movements[0]).toMatchObject({
      movement_id: 12,
      description: 'Books',
      notes: 'n1',
      category: 'Groceries',
    });
  });

  test('E3 an unchanged description is not stored', async ({ page }) => {
    const applied = await setup(page);
    await openBook(page);
    await makeDirty(page);
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    await createSelected(page, 'Book');
    await expect.poll(() => applied.length).toBe(1);
    expect(applied[0].movements[0]).not.toHaveProperty('description');
    expect(applied[0].movements[0]).toMatchObject({ notes: 'n1' });
  });

  test('E4 Previous/Next walk the Missing rows and stop at the ends', async ({ page }) => {
    await setup(page);
    await openBook(page);
    await sheet(page).getByRole('button', { name: 'Next movement' }).click();
    await expect(sheet(page)).toContainText('Movement 2 of 2');
    await expect(sheet(page).getByRole('textbox', { name: 'Description' })).toHaveValue('Salary');
    await expect(sheet(page).getByRole('button', { name: 'Next movement' })).toBeDisabled();
    await sheet(page).getByRole('button', { name: 'Previous movement' }).click();
    await expect(sheet(page)).toContainText('Movement 1 of 2');
  });

  test('E5 dirty guard on Cancel: Keep editing keeps the edit, Discard closes', async ({
    page,
  }) => {
    await setup(page);
    await openBook(page);
    await makeDirty(page);
    await sheet(page).getByRole('button', { name: 'Cancel' }).click();
    await expect(guard(page)).toBeVisible();
    await guard(page).getByRole('button', { name: 'Keep editing' }).click();
    await expect(guard(page)).toHaveCount(0);
    await expect(sheet(page).getByRole('textbox', { name: 'Notes' })).toHaveValue('n1');
    await sheet(page).getByRole('button', { name: 'Cancel' }).click();
    await guard(page).getByRole('button', { name: 'Discard' }).click();
    await expect(sheet(page)).toHaveCount(0);
  });

  test('E6 dirty guard on Next: Keep editing stays, Discard moves on', async ({ page }) => {
    await setup(page);
    await openBook(page);
    await makeDirty(page);
    await sheet(page).getByRole('button', { name: 'Next movement' }).click();
    await expect(guard(page)).toBeVisible();
    await guard(page).getByRole('button', { name: 'Keep editing' }).click();
    await expect(sheet(page)).toContainText('Movement 1 of 2');
    await sheet(page).getByRole('button', { name: 'Next movement' }).click();
    await guard(page).getByRole('button', { name: 'Discard' }).click();
    await expect(sheet(page)).toContainText('Movement 2 of 2');
    await expect(sheet(page).getByRole('textbox', { name: 'Notes' })).toHaveValue('');
  });

  test('E7 Escape: guarded when dirty, plain close and focus return when clean', async ({
    page,
  }) => {
    await setup(page);
    await openBook(page);
    await makeDirty(page);
    await page.keyboard.press('Escape');
    await expect(guard(page)).toBeVisible();
    await guard(page).getByRole('button', { name: 'Discard' }).click();
    await expect(sheet(page)).toHaveCount(0);
    await expect(editBtn(page, 'Book')).toBeFocused();

    await openBook(page);
    await page.keyboard.press('Escape');
    await expect(guard(page)).toHaveCount(0);
    await expect(sheet(page)).toHaveCount(0);
    await expect(editBtn(page, 'Book')).toBeFocused();
  });

  test('E8 bulk personalize: one sheet for the selection, fill-only apply, no prev/next', async ({
    page,
  }) => {
    const applied = await setup(page);
    // RF-08 "empty fields leave per-movement values untouched": Book already has its own category
    await openBook(page);
    await sheet(page).getByRole('combobox', { name: 'Category' }).click();
    await page.getByRole('option', { name: 'Groceries' }).click();
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    await expect(sheet(page)).toHaveCount(0);
    await page.getByRole('checkbox', { name: 'Select all' }).check();
    await page
      .getByRole('toolbar', { name: 'Selection' })
      .getByRole('button', { name: 'Personalize (2)' })
      .click();
    const bulk = page.getByRole('dialog', { name: 'Personalize 2 movements' });
    await expect(bulk).toBeVisible();
    await expect(bulk.getByRole('button', { name: 'Previous movement' })).toHaveCount(0);
    await bulk.getByRole('textbox', { name: 'Notes' }).fill('bulk');
    await bulk.getByRole('button', { name: 'Apply' }).click();
    await expect(bulk).toHaveCount(0);
    await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
    await expect.poll(() => applied.length).toBe(1);
    expect(applied[0].movements).toHaveLength(2);
    for (const m of applied[0].movements) expect(m).toMatchObject({ notes: 'bulk' });
    for (const m of applied[0].movements) expect(m).not.toHaveProperty('description');
    const byId = (id: number) => applied[0].movements.find((m) => m.movement_id === id);
    expect(byId(12), 'Book keeps its own category').toMatchObject({ category: 'Groceries' });
    expect(byId(13), 'Salary had none').not.toHaveProperty('category');
  });

  // Create must clear only the personalizations it consumed (reviewer #3).
  for (const [name, outcome] of [
    ['a row that was not selected', undefined],
    ['a row that failed', 'failed'],
  ] as const) {
    test(`E11 Create keeps the personalization of ${name}`, async ({ page }) => {
      const applied = await setup(
        page,
        results.missing,
        outcome ? { 13: outcome } : { 12: 'created' },
      );
      for (const [row, note] of [
        ['Book', 'nb'],
        ['Salary', 'ns'],
      ]) {
        await editBtn(page, row).click();
        await sheet(page).getByRole('textbox', { name: 'Notes' }).fill(note);
        await sheet(page).getByRole('button', { name: 'Save' }).click();
        await expect(sheet(page)).toHaveCount(0);
      }
      // 1st Create: Book only, or Salary only when it is the one that fails
      await createSelected(page, outcome ? 'Salary' : 'Book');
      await expect.poll(() => applied.length).toBe(1);
      await expect(page.getByRole('dialog')).toHaveCount(0);

      // 2nd Create: Salary was not created, so it carries its notes again
      await createSelected(page, 'Salary');
      await expect.poll(() => applied.length).toBe(2);
      expect(applied[1].movements).toHaveLength(1);
      expect(applied[1].movements[0]).toMatchObject({ movement_id: 13, notes: 'ns' });
    });
  }

  for (const [name, rows, expected, hidden] of [
    [
      'withdrawals only offer expense accounts',
      [BOOK, { ...BOOK, id: 15, line_no: 6, description: 'Shoes' }],
      'Supermarket',
      ['Employer', 'Checking'],
    ],
    [
      'deposits only offer revenue accounts',
      [SALARY, { ...SALARY, id: 16, line_no: 7, description: 'Bonus' }],
      'Employer',
      ['Supermarket', 'Checking'],
    ],
  ] as const) {
    test(`E8b bulk counterparty: ${name}`, async ({ page }) => {
      await setup(page, [...rows]);
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Personalize (2)' }).click();
      await page.getByRole('combobox', { name: 'Counterparty account' }).click();
      const list = page.getByRole('listbox');
      await expect(list.getByRole('option', { name: new RegExp(expected) })).toBeVisible();
      for (const h of hidden)
        await expect(list.getByRole('option', { name: new RegExp(h) })).toHaveCount(0);
    });
  }

  test('E8c bulk counterparty: mixed directions offer only the none option', async ({ page }) => {
    await setup(page);
    await page.getByRole('checkbox', { name: 'Select all' }).check();
    await page.getByRole('button', { name: 'Personalize (2)' }).click();
    await page.getByRole('combobox', { name: 'Counterparty account' }).click();
    const options = page.getByRole('listbox').getByRole('option');
    await expect(options).toHaveCount(1);
    await expect(options.first()).toHaveText('—');
  });

  test('E9 the sheet does not survive a route change', async ({ page }) => {
    await setup(page);
    await openBook(page);
    await page.evaluate(() => {
      history.pushState({}, '', '/imports');
      dispatchEvent(new PopStateEvent('popstate'));
    });
    await expect(page).toHaveURL(/\/imports$/);
    await expect(page.getByRole('dialog')).toHaveCount(0);
  });

  test.describe('at 390px', () => {
    test.use({ viewport: { width: 390, height: 844 } });

    test('E10 bottom sheet stays inside the viewport with Save reachable', async ({ page }) => {
      await setup(page);
      await openBook(page);
      await settled(sheet(page));
      const box = (await sheet(page).boundingBox())!;
      expect(box.x, 'left edge').toBeGreaterThanOrEqual(0);
      expect(box.x + box.width, 'right edge').toBeLessThanOrEqual(390);
      expect(box.y + box.height, 'docked at the bottom').toBeGreaterThan(844 - 4);
      expect(box.height, 'max 85dvh').toBeLessThanOrEqual(844 * 0.85 + 1);
      const saveBtn = sheet(page).getByRole('button', { name: 'Save' });
      const save = (await saveBtn.boundingBox())!;
      expect(save.y + save.height).toBeLessThanOrEqual(844);
      const over = await sheet(page).evaluate((d) => d.scrollWidth - d.clientWidth);
      expect(over).toBeLessThanOrEqual(0);
    });
  });
});
