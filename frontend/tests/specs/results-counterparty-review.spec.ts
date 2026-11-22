import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import {
  mockSettings,
  mockTags,
  mockCategories,
  mockBudgets,
  mockCounterpartyAccounts,
  mockCounterpartyResults,
  cpBatch,
} from '../mocks';

// Reviewer follow-ups for RF-08 v0.10 (counterparty picker): created rows show the type the backend
// reported, bulk "Leave unchanged", list stays open on inner clicks, Alt+Arrow, Firefly type labels.
const BOOK = {
  id: 12,
  line_no: 3,
  date: '2026-08-02',
  amount_cents: -9900,
  description: 'Book',
  status: 'missing',
};
const SHOES = { ...BOOK, id: 15, line_no: 6, description: 'Shoes' };

const sheet = (page: Page) => page.getByRole('dialog', { name: 'Edit movement' });
const combo = (page: Page) => sheet(page).getByRole('combobox', { name: 'Destination account' });

async function openBook(page: Page) {
  await page
    .getByRole('row')
    .filter({ hasText: 'Book' })
    .getByRole('button', { name: 'Edit' })
    .click();
  await expect(sheet(page)).toBeVisible();
}

test.beforeEach(async ({ page, mockImportsPage }) => {
  await mockImportsPage();
  await mockSettings(page);
  await mockTags(page);
  await mockCategories(page);
  await mockBudgets(page);
});

test.describe('Results > created rows show the backend transaction_type', () => {
  // After Create the row leaves Missing and appears in Matched, where the report's type is shown.
  async function createAndOpenMatched(page: Page, type: string, locale = 'en') {
    await page.addInitScript((l) => localStorage.setItem('ffm.locale', l), locale);
    await mockCounterpartyAccounts(page);
    let created = false;
    await page.route('**/api/imports/1', (r) =>
      r.fulfill({ json: { batch: cpBatch, movements: [], headers: [], detected_delimiter: ',' } }),
    );
    await page.route('**/api/imports/1/results', (r) =>
      r.fulfill({
        json: {
          reconciliation_id: 1,
          matched: created
            ? [
                {
                  movement: { ...BOOK, status: 'matched' },
                  ff3: {
                    id: 7,
                    ff3_id: 107,
                    date: '2026-08-02',
                    amount_cents: -9900,
                    description: 'Book',
                    is_duplicate: false,
                    created_at: '2026-08-02T00:00:00Z',
                    counterparty_name: 'Rainy Day',
                    counterparty_type: 'Asset account',
                  },
                },
              ]
            : [],
          missing: created ? [] : [BOOK],
          ignored: [],
          extra: [],
        },
      }),
    );
    await page.route('**/api/imports/1/apply', async (r) => {
      created = true;
      await r.fulfill({
        json: {
          reports: [{ movement_id: 12, outcome: 'created', transaction_type: type }],
          created_count: 1,
          already_created_count: 0,
          failed_count: 0,
        },
      });
    });
    await page.goto('/results/1');
    await page.getByRole('row').filter({ hasText: 'Book' }).getByRole('checkbox').check();
    await page.getByRole('button', { name: /Firefly III \(1\)$/ }).click();
    await page
      .getByRole('dialog')
      .getByRole('button', { name: /^(Create|Crear|Créer)$/ })
      .click();
    await page.getByRole('tab', { name: /Matched|Conciliados|Rapprochés/ }).click();
    return page.getByRole('row').filter({ hasText: 'Book' });
  }

  for (const [type, label] of [
    ['transfer', 'Transfer'],
    ['withdrawal', 'Withdrawal'],
    ['deposit', 'Deposit'],
  ] as const) {
    test(`R1 a just-created row states «${label}» (backend type ${type}) in Counterparty`, async ({
      page,
    }) => {
      const row = await createAndOpenMatched(page, type);
      await expect(row).toContainText(label);
      await expect(row).toContainText('Rainy Day');
    });
  }

  for (const [locale, label] of [
    ['es', 'Transferencia'],
    ['fr', 'Transfert'],
  ] as const) {
    test(`R1 the created type is translated (${locale})`, async ({ page }) => {
      const row = await createAndOpenMatched(page, 'transfer', locale);
      await expect(row).toContainText(label);
    });
  }
});

test.describe('Bulk "Leave unchanged"', () => {
  const bulkOpen = async (page: Page) => {
    await page.getByRole('checkbox', { name: 'Select all' }).check();
    await page.getByRole('button', { name: 'Personalize (2)' }).click();
    return page.getByRole('dialog', { name: 'Personalize 2 movements' });
  };

  test("R2 it returns to the initial state and keeps each row's own counterparty", async ({
    page,
  }) => {
    const applied = await mockCounterpartyResults(page, [BOOK, SHOES]);
    await page.goto('/results/1');
    await openBook(page);
    await combo(page).click();
    await page.getByRole('option', { name: /Supermarket/ }).click();
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    await expect(sheet(page)).toHaveCount(0);

    const bulk = await bulkOpen(page);
    const input = bulk.getByRole('combobox', { name: 'Destination account' });
    await input.click();
    await page.getByRole('option', { name: /Rainy Day/ }).click();
    await expect(bulk.getByText('Will be created as a transfer')).toBeVisible();

    await input.click();
    await page.getByRole('listbox').getByRole('option', { name: 'Leave unchanged' }).click();
    await expect(input).toHaveValue('');
    await expect(input).toHaveAttribute('placeholder', 'Leave unchanged');
    await expect(bulk.getByText(/Will be created as a/)).toHaveCount(0);
    await bulk.getByRole('button', { name: 'Apply' }).click();

    await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
    await page.getByRole('dialog').getByRole('button', { name: 'Create', exact: true }).click();
    await expect.poll(() => applied.length).toBe(1);
    const byId = (id: number) => applied[0].movements.find((m) => m.movement_id === id)!;
    expect(byId(12), 'Book keeps its own account').toMatchObject({ counterparty_ff3_id: 51 });
    expect(byId(15)).not.toHaveProperty('counterparty_ff3_id');
    expect(byId(15)).not.toHaveProperty('counterparty_name');
  });

  test('R2b the option only exists in bulk, after Cash', async ({ page }) => {
    await mockCounterpartyResults(page, [BOOK, SHOES]);
    await page.goto('/results/1');
    await openBook(page);
    await combo(page).click();
    await expect(page.getByRole('option', { name: 'Leave unchanged' })).toHaveCount(0);
    await page.keyboard.press('Escape');
    await page.keyboard.press('Escape');
    await expect(sheet(page)).toHaveCount(0);
    const bulk = await bulkOpen(page);
    await bulk.getByRole('combobox', { name: 'Destination account' }).click();
    const names = await page.getByRole('listbox').getByRole('option').allTextContents();
    expect(names.at(-2)).toContain('Cash / no counterparty');
    expect(names.at(-1)).toContain('Leave unchanged');
  });

  for (const [locale, dest, keep] of [
    ['es', 'Cuenta destino', 'Dejar sin cambios'],
    ['fr', 'Compte de destination', 'Laisser inchangé'],
  ] as const) {
    test(`R2c placeholder and option are translated (${locale})`, async ({ page }) => {
      await page.addInitScript((l) => localStorage.setItem('ffm.locale', l), locale);
      await mockCounterpartyResults(page, [BOOK, SHOES]);
      await page.goto('/results/1');
      await page.getByRole('checkbox').first().check();
      await page
        .getByRole('button', { name: /^(Personalize|Personalizar|Personnaliser).*\(2\)$/ })
        .first()
        .click();
      const input = page.getByRole('dialog').getByRole('combobox', { name: dest });
      await expect(input).toHaveAttribute('placeholder', keep);
      await input.click();
      await expect(page.getByRole('listbox').getByRole('option', { name: keep })).toBeVisible();
    });
  }
});

test.describe('Picker list behaviour', () => {
  test('R3 a click inside the list (group title, padding) does not close it', async ({ page }) => {
    await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    await openBook(page);
    await combo(page).click();
    const list = page.getByRole('listbox');
    await list.getByText('Expense accounts', { exact: true }).click();
    await expect(list).toBeVisible();
    await expect(combo(page)).toBeFocused();
    await list.click({ position: { x: 1, y: 1 } });
    await expect(list).toBeVisible();
    await expect(combo(page)).toBeFocused();
  });

  test('R6 Alt+ArrowUp closes the list, Alt+ArrowDown opens it; value, focus and sheet stay', async ({
    page,
  }) => {
    await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    await openBook(page);
    await combo(page).click();
    await page.getByRole('option', { name: /Supermarket/ }).click();
    await expect(combo(page)).toHaveValue(/Supermarket/);
    await combo(page).focus();
    await page.keyboard.press('Alt+ArrowDown');
    await expect(page.getByRole('listbox')).toBeVisible();
    await page.keyboard.press('Alt+ArrowDown'); // already open: no-op
    await expect(page.getByRole('listbox')).toBeVisible();
    await page.keyboard.press('Alt+ArrowUp');
    await expect(page.getByRole('listbox')).toHaveCount(0);
    await expect(combo(page)).toBeFocused();
    await expect(combo(page)).toHaveValue(/Supermarket/);
    await expect(sheet(page)).toBeVisible();
  });
});

test.describe('Dashboard > Browse: Firefly special account types are translated', () => {
  const types: [string, Record<string, string>][] = [
    [
      'Initial balance account',
      { en: 'Initial balance', es: 'Saldo inicial', fr: 'Solde initial' },
    ],
    ['Reconciliation account', { en: 'Reconciliation', es: 'Conciliación', fr: 'Rapprochement' }],
    [
      'Liability credit account',
      { en: 'Liability credit', es: 'Crédito de pasivo', fr: 'Crédit de passif' },
    ],
  ];
  const accountLabel = {
    en: 'Firefly III account',
    es: 'Cuenta de Firefly III',
    fr: 'Compte Firefly III',
  };

  for (const locale of ['en', 'es', 'fr'] as const) {
    test(`R4 badges show the ${locale} label, never the raw Firefly literal`, async ({
      page,
      mockDashboard,
    }) => {
      await page.addInitScript((l) => localStorage.setItem('ffm.locale', l), locale);
      await page.clock.setFixedTime(new Date('2026-09-09T12:00:00Z'));
      await mockDashboard();
      await page.route('**/api/firefly/accounts/*/transactions*', (r) =>
        r.fulfill({
          json: types.map(([ft], i) => ({
            id: 200 + i,
            date: `2026-09-0${i + 1}`,
            amount_cents: -100,
            description: `Tx${i}`,
            status: null,
            counterparty_name: `Acct${i}`,
            counterparty_type: ft,
          })),
        }),
      );
      await page.goto('/');
      await page.getByRole('combobox', { name: accountLabel[locale] }).click();
      await page.getByRole('option', { name: 'Checking' }).click();
      for (const [i, [ft, labels]] of types.entries()) {
        const row = page.getByRole('listitem').filter({ hasText: `Tx${i}` });
        await expect(row).toContainText(`Acct${i}`);
        await expect(row).toContainText(labels[locale]);
        await expect(row).not.toContainText(ft);
      }
    });
  }
});
