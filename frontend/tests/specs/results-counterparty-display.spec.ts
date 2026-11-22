import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import { DashboardPage } from '../pages/DashboardPage';
import { fixtures } from '../fixtures/fixtures';
import { mockCounterpartyAccounts } from '../mocks';

// RF-07 / RF-14 v0.10: the counterparty of a Firefly III transaction (name + type badge) shows in
// Matched and in Browse; old snapshots (null) leave the cell empty.
const ff3 = (id: number, description: string, counterparty: [string, string] | null) => ({
  id,
  ff3_id: 100 + id,
  date: '2026-08-01',
  amount_cents: -4200,
  description,
  is_duplicate: false,
  created_at: '2026-08-01T00:00:00Z',
  counterparty_name: counterparty?.[0] ?? null,
  counterparty_type: counterparty?.[1] ?? null,
});
const mv = (id: number, description: string) => ({
  id,
  line_no: id,
  date: '2026-08-01',
  amount_cents: -4200,
  description,
  status: 'matched',
});

const matched = [
  { movement: mv(1, 'Coffee'), ff3: ff3(1, 'Coffee', ['Supermarket', 'Expense account']) },
  { movement: mv(2, 'Move'), ff3: ff3(2, 'Move', ['Rainy Day', 'Asset account']) },
  { movement: mv(3, 'Loan pay'), ff3: ff3(3, 'Loan pay', ['Car Loan', 'Loan']) },
  { movement: mv(4, 'Old row'), ff3: ff3(4, 'Old row', null) },
];

test.describe('Results > Matched counterparty column', () => {
  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockCounterpartyAccounts(page);
    await page.route('**/api/imports/1', (r) =>
      r.fulfill({
        json: { batch: fixtures.imports[0], movements: [], headers: [], detected_delimiter: ',' },
      }),
    );
    await page.route('**/api/imports/1/results', (r) =>
      r.fulfill({
        json: { reconciliation_id: 1, matched, missing: [], ignored: [], extra: [] },
      }),
    );
  });

  const table = (page: Page) => page.getByRole('table', { name: 'Matched' });
  const row = (page: Page, text: string) => table(page).getByRole('row').filter({ hasText: text });

  test('D1 shows the counterparty name and a type badge per row', async ({ page }) => {
    await page.goto('/results/1');
    await page.getByRole('tab', { name: /Matched/ }).click();
    await expect(table(page).getByRole('columnheader', { name: 'Counterparty' })).toBeVisible();
    await expect(row(page, 'Coffee')).toContainText('Supermarket');
    await expect(row(page, 'Coffee')).toContainText('Expense');
    await expect(row(page, 'Move')).toContainText('Rainy Day');
    await expect(row(page, 'Move')).toContainText('Own account');
    await expect(row(page, 'Loan pay')).toContainText('Car Loan');
    await expect(row(page, 'Loan pay').getByText('Loan', { exact: true })).toBeVisible();
  });

  test('D2 null counterparty (old snapshot) leaves the cell empty, not a placeholder', async ({
    page,
  }) => {
    await page.goto('/results/1');
    await page.getByRole('tab', { name: /Matched/ }).click();
    const headers = await table(page)
      .getByRole('columnheader')
      .evaluateAll((hs) => hs.map((h) => h.textContent?.trim()));
    const col = headers.indexOf('Counterparty');
    expect(col, 'Counterparty column exists').toBeGreaterThanOrEqual(0);
    const cell = row(page, 'Old row').getByRole('cell').nth(col);
    await expect(cell).toBeEmpty();
    await expect(row(page, 'Old row')).not.toContainText('—');
  });

  for (const [locale, header, badge] of [
    ['es', 'Contrapartida', 'Gasto'],
    ['fr', 'Contrepartie', 'Dépense'],
  ] as const) {
    test(`D3 header and badge are translated (${locale})`, async ({ page }) => {
      await page.addInitScript((l) => localStorage.setItem('ffm.locale', l), locale);
      await page.goto('/results/1');
      await page.getByRole('tab', { name: /Conciliad|Rapproch/i }).click();
      await expect(page.getByRole('columnheader', { name: header })).toBeVisible();
      await expect(page.getByRole('row').filter({ hasText: 'Coffee' })).toContainText(badge);
    });
  }
});

test.describe('Dashboard > Browse counterparty', () => {
  const browseTxs = [
    {
      id: 42,
      date: '2026-09-01',
      amount_cents: -4200,
      description: 'Coffee',
      status: 'matched',
      counterparty_name: 'Supermarket',
      counterparty_type: 'Expense account',
    },
    {
      id: 43,
      date: '2026-09-03',
      amount_cents: -1500,
      description: 'Card payment',
      status: null,
      counterparty_name: 'Visa Card',
      counterparty_type: 'Asset account',
    },
    {
      id: 44,
      date: '2026-09-05',
      amount_cents: 250000,
      description: 'Salary',
      status: null,
      counterparty_name: null,
      counterparty_type: null,
    },
  ];

  test.beforeEach(async ({ page, mockDashboard }) => {
    await page.clock.setFixedTime(new Date('2026-09-09T12:00:00Z'));
    await mockDashboard();
    await page.route('**/api/firefly/accounts/*/transactions*', (r) =>
      r.fulfill({ json: browseTxs }),
    );
  });

  test('D4 each row shows the counterparty name and type badge', async ({ page }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();
    await dashboard.selectAccount('Checking');
    const rows = dashboard.browseList.getByRole('listitem');
    await expect(rows).toHaveCount(3);
    await expect(rows.filter({ hasText: 'Coffee' })).toContainText('Supermarket');
    await expect(rows.filter({ hasText: 'Coffee' })).toContainText('Expense');
    await expect(rows.filter({ hasText: 'Card payment' })).toContainText('Visa Card');
    await expect(rows.filter({ hasText: 'Card payment' })).toContainText('Own account');
  });

  test('D5 a null counterparty shows no name and no badge', async ({ page }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();
    await dashboard.selectAccount('Checking');
    const salary = dashboard.browseList.getByRole('listitem').filter({ hasText: 'Salary' });
    await expect(salary).toBeVisible();
    for (const t of ['Supermarket', 'Visa Card', 'Expense', 'Own account', 'Revenue', 'Cash'])
      await expect(salary).not.toContainText(t);
  });
});
