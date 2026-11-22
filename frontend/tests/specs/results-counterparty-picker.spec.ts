import type { Locator, Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import {
  mockSettings,
  mockTags,
  mockCategories,
  mockBudgets,
  mockCounterpartyResults,
  type CpApplyBody,
} from '../mocks';

// RF-08 v0.10: the counterparty picker is a grouped, searchable combobox per direction;
// the resulting Firefly III type follows from the counterparty (plan table).
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
const SHOES = { ...BOOK, id: 15, line_no: 6, description: 'Shoes' };
const BONUS = { ...SALARY, id: 16, line_no: 7, description: 'Bonus' };

const sheet = (page: Page) => page.getByRole('dialog', { name: 'Edit movement' });
const picker = (page: Page, label: string) => sheet(page).getByRole('combobox', { name: label });
const typeLine = (page: Page, kind: string) =>
  sheet(page).getByText(`Will be created as a ${kind}`, { exact: true });

async function openSheet(page: Page, row: string) {
  await page
    .getByRole('row')
    .filter({ hasText: row })
    .getByRole('button', { name: 'Edit' })
    .click();
  await expect(sheet(page)).toBeVisible();
}

async function openPicker(page: Page, row: 'Book' | 'Salary') {
  await openSheet(page, row);
  await picker(page, row === 'Book' ? 'Destination account' : 'Source account').click();
  return page.getByRole('listbox');
}

const group = (list: Locator, name: string) => list.getByRole('group', { name, exact: true });

/** Edit one row, pick an option, Save, then Create it and return what the app sent. */
async function pickAndCreate(
  page: Page,
  applied: { movements: Record<string, unknown>[] }[],
  row: 'Book' | 'Salary',
  option: string | RegExp,
  search?: string,
) {
  await openSheet(page, row);
  await picker(page, row === 'Book' ? 'Destination account' : 'Source account').click();
  if (search) await page.keyboard.type(search);
  await page.getByRole('option', { name: option }).click();
  await sheet(page).getByRole('button', { name: 'Save' }).click();
  await expect(sheet(page)).toHaveCount(0);
  await page.getByRole('row').filter({ hasText: row }).getByRole('checkbox').check();
  await page.getByRole('button', { name: 'Create in Firefly III (1)' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
  await expect.poll(() => applied.length).toBe(1);
  // RF-08 v0.10: the type is derived on the backend, never sent by the client
  for (const k of ['transaction_type', 'type'])
    expect(applied[0].movements[0], `client must not send ${k}`).not.toHaveProperty(k);
  return applied[0].movements[0];
}

test.describe('Results > counterparty picker', () => {
  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
  });

  test('C1 money out groups: expense, liabilities, own accounts; never revenue nor the batch account', async ({
    page,
  }) => {
    await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    const list = await openPicker(page, 'Book');
    await expect(group(list, 'Expense accounts').getByRole('option')).toHaveText([
      /Supermarket/,
      /Pizza Place/,
    ]);
    await expect(group(list, 'Liabilities').getByRole('option')).toHaveText([
      /Car Loan/,
      /House Mortgage/,
    ]);
    const own = group(list, 'Own accounts').getByRole('option');
    await expect(own.filter({ hasText: 'Rainy Day' })).toHaveCount(1);
    await expect(own.filter({ hasText: 'Visa Card' })).toContainText('Credit card');
    await expect(own.filter({ hasText: 'Rainy Day' })).not.toContainText('Credit card');
    await expect(list.getByRole('group', { name: 'Revenue accounts' })).toHaveCount(0);
    for (const hidden of ['Employer', 'Refunds Inc', 'Checking', 'Savings USD'])
      await expect(list.getByRole('option', { name: new RegExp(hidden) })).toHaveCount(0);
  });

  test('C2 money in groups: revenue, liabilities, own accounts; never expense nor the batch account', async ({
    page,
  }) => {
    await mockCounterpartyResults(page, [SALARY]);
    await page.goto('/results/1');
    const list = await openPicker(page, 'Salary');
    await expect(group(list, 'Revenue accounts').getByRole('option')).toHaveText([
      /Employer/,
      /Refunds Inc/,
    ]);
    await expect(group(list, 'Liabilities').getByRole('option')).toHaveCount(2);
    await expect(
      group(list, 'Own accounts').getByRole('option', { name: /Visa Card/ }),
    ).toContainText('Credit card');
    await expect(list.getByRole('group', { name: 'Expense accounts' })).toHaveCount(0);
    for (const hidden of ['Supermarket', 'Pizza Place', 'Checking', 'Savings USD'])
      await expect(list.getByRole('option', { name: new RegExp(hidden) })).toHaveCount(0);
  });

  test('C3 the search filters options and hides empty groups', async ({ page }) => {
    await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    const list = await openPicker(page, 'Book');
    await page.keyboard.type('car');
    await expect(list.getByRole('option', { name: /Car Loan/ })).toBeVisible();
    await expect(list.getByRole('option', { name: /Supermarket/ })).toHaveCount(0);
    await expect(list.getByRole('group', { name: 'Expense accounts' })).toHaveCount(0);
    // Visa Card matches "car" too (case-insensitive substring)
    await expect(list.getByRole('option', { name: /Visa Card/ })).toBeVisible();
  });

  test('C4 keyboard: arrows move, Enter picks, Escape closes the list but not the sheet', async ({
    page,
  }) => {
    await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    await openPicker(page, 'Book');
    const combo = picker(page, 'Destination account');
    const active = () => combo.getAttribute('aria-activedescendant');
    await page.keyboard.press('ArrowDown');
    const first = await active();
    await page.keyboard.press('ArrowDown');
    expect(await active(), 'ArrowDown moves the active option').not.toBe(first);
    await page.keyboard.press('ArrowUp');
    expect(await active()).toBe(first);

    await page.keyboard.type('pizza');
    // walk with the arrows until the active option is the account (order of the extras is not the point)
    const activeText = async () => {
      const id = await active();
      return id ? ((await page.locator(`[id="${id}"]`).textContent()) ?? '') : '';
    };
    for (let i = 0; i < 4 && !/Pizza Place/.test(await activeText()); i++)
      await page.keyboard.press('ArrowDown');
    await expect.poll(activeText).toMatch(/Pizza Place/);
    await page.keyboard.press('Enter');
    await expect(page.getByRole('listbox')).toHaveCount(0);
    await expect(combo).toHaveValue(/Pizza Place/);
    await expect(typeLine(page, 'withdrawal')).toBeVisible();

    await sheet(page).getByRole('textbox', { name: 'Notes' }).click();
    await combo.click();
    await expect(page.getByRole('listbox')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('listbox')).toHaveCount(0);
    await expect(sheet(page)).toBeVisible();
    await expect(combo).toBeFocused();
  });

  for (const [locale, out, inn] of [
    ['en', 'Destination account', 'Source account'],
    ['es', 'Cuenta destino', 'Cuenta origen'],
    ['fr', 'Compte de destination', 'Compte source'],
  ] as const) {
    test(`C5 label follows the direction (${locale})`, async ({ page }) => {
      await page.addInitScript((l) => localStorage.setItem('ffm.locale', l), locale);
      await mockCounterpartyResults(page, [BOOK, SALARY]);
      await page.goto('/results/1');
      const edit = { en: 'Edit', es: 'Editar', fr: 'Modifier' }[locale];
      const row = (text: string) =>
        page.getByRole('row').filter({ hasText: text }).getByRole('button', { name: edit });
      await row('Book').click();
      await expect(page.getByRole('dialog').getByRole('combobox', { name: out })).toBeVisible();
      await expect(page.getByRole('dialog').getByRole('combobox', { name: inn })).toHaveCount(0);
      await page.keyboard.press('Escape');
      await row('Salary').click();
      await expect(page.getByRole('dialog').getByRole('combobox', { name: inn })).toBeVisible();
      await expect(page.getByRole('dialog').getByRole('combobox', { name: out })).toHaveCount(0);
    });
  }

  test.describe('apply payload', () => {
    test('C6 an existing account sends counterparty_ff3_id and no name', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK], {
        12: { outcome: 'created', transaction_type: 'withdrawal' },
      });
      await page.goto('/results/1');
      const m = await pickAndCreate(page, applied, 'Book', /Supermarket/);
      expect(m).toMatchObject({ movement_id: 12, counterparty_ff3_id: 51 });
      expect(m).not.toHaveProperty('counterparty_name');
    });

    test('C7 «Create X» sends counterparty_name and no id (out and in)', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK], {
        12: { outcome: 'created', transaction_type: 'withdrawal' },
      });
      await page.goto('/results/1');
      const out = await pickAndCreate(page, applied, 'Book', /Create .*Bookshop/, 'Bookshop');
      expect(out).toMatchObject({ movement_id: 12, counterparty_name: 'Bookshop' });
      expect(out).not.toHaveProperty('counterparty_ff3_id');
    });

    test('C7b «Create X» on money in names a revenue account', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [SALARY], {
        13: { outcome: 'created', transaction_type: 'deposit' },
      });
      await page.goto('/results/1');
      await openPicker(page, 'Salary');
      await page.keyboard.type('Acme');
      await expect(
        page.getByRole('option', { name: /Create .*Acme.* as revenue account/ }),
      ).toBeVisible();
      await expect(page.getByRole('option', { name: /as expense account/ })).toHaveCount(0);
      await page.getByRole('option', { name: /Create .*Acme/ }).click();
      await sheet(page).getByRole('button', { name: 'Save' }).click();
      await page.getByRole('row').filter({ hasText: 'Salary' }).getByRole('checkbox').check();
      await page.getByRole('button', { name: 'Create in Firefly III (1)' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
      await expect.poll(() => applied.length).toBe(1);
      expect(applied[0].movements[0]).toMatchObject({ movement_id: 13, counterparty_name: 'Acme' });
    });

    test('C7c no «Create X» offer when the typed name is an existing account', async ({ page }) => {
      await mockCounterpartyResults(page, [BOOK]);
      await page.goto('/results/1');
      await openPicker(page, 'Book');
      await page.keyboard.type('Supermarket');
      await expect(page.getByRole('option', { name: /Supermarket/ })).toBeVisible();
      await expect(page.getByRole('option', { name: /Create/ })).toHaveCount(0);
    });

    test('C8 Cash sends neither counterparty_ff3_id nor counterparty_name', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK], {
        12: { outcome: 'created', transaction_type: 'withdrawal' },
      });
      await page.goto('/results/1');
      const m = await pickAndCreate(page, applied, 'Book', /Cash/);
      expect(m).toMatchObject({ movement_id: 12 });
      expect(m).not.toHaveProperty('counterparty_ff3_id');
      expect(m).not.toHaveProperty('counterparty_name');
    });

    test('C8b picking an account and then Cash clears the earlier choice', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK], {
        12: { outcome: 'created', transaction_type: 'withdrawal' },
      });
      await page.goto('/results/1');
      await openPicker(page, 'Book');
      await page.getByRole('option', { name: /Supermarket/ }).click();
      await picker(page, 'Destination account').click();
      await page.getByRole('option', { name: /Cash/ }).click();
      await sheet(page).getByRole('button', { name: 'Save' }).click();
      await page.getByRole('row').filter({ hasText: 'Book' }).getByRole('checkbox').check();
      await page.getByRole('button', { name: 'Create in Firefly III (1)' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
      await expect.poll(() => applied.length).toBe(1);
      expect(applied[0].movements[0]).not.toHaveProperty('counterparty_ff3_id');
      expect(applied[0].movements[0]).not.toHaveProperty('counterparty_name');
    });
  });

  // One case per row of the plan table; the line under the picker states the resulting type.
  for (const [name, row, option, search, kind] of [
    ['out · expense account', 'Book', /Supermarket/, undefined, 'withdrawal'],
    ['out · liability', 'Book', /Car Loan/, undefined, 'withdrawal'],
    ['out · own asset', 'Book', /Rainy Day/, undefined, 'transfer'],
    ['out · own credit card', 'Book', /Visa Card/, undefined, 'transfer'],
    ['out · new name', 'Book', /Create .*Bookshop/, 'Bookshop', 'withdrawal'],
    ['out · Cash', 'Book', /Cash/, undefined, 'withdrawal'],
    ['in · revenue account', 'Salary', /Employer/, undefined, 'deposit'],
    ['in · liability', 'Salary', /House Mortgage/, undefined, 'deposit'],
    ['in · own asset', 'Salary', /Rainy Day/, undefined, 'transfer'],
    ['in · new name', 'Salary', /Create .*Acme/, 'Acme', 'deposit'],
    ['in · Cash', 'Salary', /Cash/, undefined, 'deposit'],
  ] as const) {
    test(`C9 type line: ${name} → ${kind}`, async ({ page }) => {
      await mockCounterpartyResults(page, [BOOK, SALARY]);
      await page.goto('/results/1');
      await openPicker(page, row);
      if (search) await page.keyboard.type(search);
      await page.getByRole('option', { name: option }).click();
      await expect(typeLine(page, kind)).toBeVisible();
      // exactly one type is stated at a time
      await expect(sheet(page).getByText(/Will be created as a/)).toHaveCount(1);
    });
  }

  test('C9b the type line updates when the choice changes', async ({ page }) => {
    await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    await openPicker(page, 'Book');
    await page.getByRole('option', { name: /Rainy Day/ }).click();
    await expect(typeLine(page, 'transfer')).toBeVisible();
    await picker(page, 'Destination account').click();
    await page.getByRole('option', { name: /Supermarket/ }).click();
    await expect(typeLine(page, 'withdrawal')).toBeVisible();
    await expect(typeLine(page, 'transfer')).toHaveCount(0);
  });

  test('C10 the confirm dialog states the type and the account of each row', async ({ page }) => {
    await mockCounterpartyResults(page, [BOOK, SALARY, SHOES]);
    await page.goto('/results/1');
    for (const [row, option, search] of [
      ['Book', /Rainy Day/, undefined],
      ['Salary', /Create .*Acme/, 'Acme'],
    ] as const) {
      await openPicker(page, row);
      if (search) await page.keyboard.type(search);
      await page.getByRole('option', { name: option }).click();
      await sheet(page).getByRole('button', { name: 'Save' }).click();
      await expect(sheet(page)).toHaveCount(0);
    }
    await page.getByRole('checkbox', { name: 'Select all' }).check();
    await page.getByRole('button', { name: 'Create in Firefly III (3)' }).click();
    const dlg = page.getByRole('dialog');
    await expect(dlg.getByText('Transfer · Rainy Day')).toBeVisible();
    await expect(dlg.getByText('Deposit · New revenue account “Acme”')).toBeVisible();
    // no counterparty picked → Cash, still a withdrawal
    await expect(dlg.getByText('Withdrawal · Cash')).toBeVisible();
  });

  test.describe('bulk', () => {
    test('C11 one direction: same grouped picker, applied to every selected row', async ({
      page,
    }) => {
      const applied = await mockCounterpartyResults(page, [BOOK, SHOES], {
        12: { outcome: 'created', transaction_type: 'transfer' },
        15: { outcome: 'created', transaction_type: 'transfer' },
      });
      await page.goto('/results/1');
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Personalize (2)' }).click();
      const bulk = page.getByRole('dialog', { name: 'Personalize 2 movements' });
      const bulkCombo = bulk.getByRole('combobox', { name: 'Destination account' });
      await expect(bulkCombo).toHaveAttribute('placeholder', 'Leave unchanged');
      await expect(bulk.getByText(/Will be created as a/)).toHaveCount(0);
      await bulkCombo.click();
      const list = page.getByRole('listbox');
      await expect(group(list, 'Expense accounts').getByRole('option')).toHaveCount(2);
      await expect(group(list, 'Liabilities').getByRole('option')).toHaveCount(2);
      await expect(group(list, 'Own accounts').getByRole('option')).toHaveCount(2);
      await expect(list.getByRole('option', { name: /Checking|Employer/ })).toHaveCount(0);
      await list.getByRole('option', { name: /Rainy Day/ }).click();
      await expect(bulk.getByText('Will be created as a transfer', { exact: true })).toBeVisible();
      await bulk.getByRole('button', { name: 'Apply' }).click();
      await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
      await expect.poll(() => applied.length).toBe(1);
      expect(applied[0].movements).toHaveLength(2);
      for (const m of applied[0].movements) expect(m).toMatchObject({ counterparty_ff3_id: 3 });
    });

    test('C11b money in: revenue group, not expense', async ({ page }) => {
      await mockCounterpartyResults(page, [SALARY, BONUS]);
      await page.goto('/results/1');
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Personalize (2)' }).click();
      await page.getByRole('combobox', { name: 'Source account' }).click();
      const list = page.getByRole('listbox');
      await expect(group(list, 'Revenue accounts').getByRole('option')).toHaveCount(2);
      await expect(list.getByRole('group', { name: 'Expense accounts' })).toHaveCount(0);
    });

    test('C11e Cash in bulk overrides a per-row choice on every selected row', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK, SHOES]);
      await page.goto('/results/1');
      await openPicker(page, 'Book');
      await page.getByRole('option', { name: /Supermarket/ }).click();
      await sheet(page).getByRole('button', { name: 'Save' }).click();
      await expect(sheet(page)).toHaveCount(0);
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Personalize (2)' }).click();
      const bulk = page.getByRole('dialog', { name: 'Personalize 2 movements' });
      await bulk.getByRole('combobox', { name: 'Destination account' }).click();
      await page.getByRole('option', { name: /Cash/ }).click();
      await bulk.getByRole('button', { name: 'Apply' }).click();
      await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
      await expect.poll(() => applied.length).toBe(1);
      for (const m of applied[0].movements) {
        expect(m).not.toHaveProperty('counterparty_ff3_id');
        expect(m).not.toHaveProperty('counterparty_name');
      }
    });

    test('C11d mixed directions: no picker, a hint to set it per movement', async ({ page }) => {
      await mockCounterpartyResults(page, [BOOK, SALARY]);
      await page.goto('/results/1');
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Personalize (2)' }).click();
      const bulk = page.getByRole('dialog', { name: 'Personalize 2 movements' });
      await expect(
        bulk.getByText('Mixed directions: set the account on each movement.'),
      ).toBeVisible();
      await expect(
        bulk.getByRole('combobox', { name: /Destination account|Source account/ }),
      ).toHaveCount(0);
    });

    test('C11c bulk «Create X» names the same new account for every row', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK, SHOES]);
      await page.goto('/results/1');
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Personalize (2)' }).click();
      const bulk = page.getByRole('dialog', { name: 'Personalize 2 movements' });
      await bulk.getByRole('combobox', { name: 'Destination account' }).click();
      await page.keyboard.type('Bookshop');
      await page.getByRole('option', { name: /Create .*Bookshop/ }).click();
      await bulk.getByRole('button', { name: 'Apply' }).click();
      await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
      await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
      await expect.poll(() => applied.length).toBe(1);
      for (const m of applied[0].movements) {
        expect(m).toMatchObject({ counterparty_name: 'Bookshop' });
        expect(m).not.toHaveProperty('counterparty_ff3_id');
      }
    });
  });

  test('C12 a rejected movement shows its reason, stays in Missing, and a valid retry creates it', async ({
    page,
  }) => {
    const reason = 'A revenue account cannot be the destination of a withdrawal';
    const applied = await mockCounterpartyResults(page, [BOOK, SHOES], {
      12: { outcome: 'failed', reason, transaction_type: null },
      15: { outcome: 'created', transaction_type: 'withdrawal' },
    });
    await page.goto('/results/1');
    await page.getByRole('checkbox', { name: 'Select all' }).check();
    await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
    await expect.poll(() => applied.length).toBe(1);
    const book = page.getByRole('row').filter({ hasText: 'Book' });
    await expect(book).toContainText(reason);
    await expect(book.getByRole('checkbox')).toBeEnabled();
    // the rest of the batch was not blocked
    await expect(page.getByRole('row').filter({ hasText: 'Shoes' })).not.toContainText(reason);

    // fix the counterparty and retry only the rejected row
    await page.unroute('**/api/imports/1/apply');
    const retry: CpApplyBody[] = [];
    await page.route('**/api/imports/1/apply', async (r) => {
      retry.push(r.request().postDataJSON() as CpApplyBody);
      await r.fulfill({
        json: {
          reports: [{ movement_id: 12, outcome: 'created', transaction_type: 'withdrawal' }],
          created_count: 1,
          already_created_count: 0,
          failed_count: 0,
        },
      });
    });
    await openSheet(page, 'Book');
    await picker(page, 'Destination account').click();
    await page.getByRole('option', { name: /Supermarket/ }).click();
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    await book.getByRole('checkbox').check();
    await page.getByRole('button', { name: 'Create in Firefly III (1)' }).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
    await expect.poll(() => retry.length).toBe(1);
    expect(retry[0].movements).toHaveLength(1);
    expect(retry[0].movements[0]).toMatchObject({ movement_id: 12, counterparty_ff3_id: 51 });
  });
});
