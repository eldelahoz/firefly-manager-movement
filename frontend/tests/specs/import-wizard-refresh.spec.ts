import type { Page, Route } from '@playwright/test';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import { test, expect } from '../fixtures/api.fixture';
import { ImportsPage } from '../pages/ImportsPage';
import { ImportWizardPage } from '../pages/ImportWizardPage';
import { fixtures } from '../fixtures/fixtures';

// M4: wizard refresh (file card, sample values, preview, invalid rows, tolerance, 390).
function csvPath(name: string, content: string | Buffer): string {
  const p = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'ffm-e2e-')), name);
  fs.writeFileSync(p, content);
  return p;
}
const CSV = 'date,amount,desc\n2026-08-01,-10,Coffee\n2026-08-02,-20,Book\n';
const CSV_UNMAPPED = 'Fecha,Valor,Desc\n05/01/2026,-10,Coffee\n';

const autoMapped = {
  batch_id: 10,
  filename: 'bank.csv',
  file_hash: 'h10',
  headers: ['date', 'amount', 'desc'],
  detected_delimiter: ',',
  total_rows: 3,
  valid_count: 2,
  invalid_count: 1,
  mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
  origin_account_values: [],
  valid: [
    { line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' },
    { line_no: 3, date: '2026-08-02', amount_cents: -2000, description: 'Book' },
  ],
  invalid: [{ line_no: 4, reason: 'invalid amount', raw: '2026-08-03,abc,Broken' }],
};
const unmapped = {
  ...autoMapped,
  batch_id: 11,
  filename: 'unknown.csv',
  headers: ['Fecha', 'Valor', 'Desc'],
  total_rows: 1,
  valid_count: 0,
  invalid_count: 0,
  mapping_used: null,
  valid: [],
  invalid: [],
};

async function mockUpload(page: Page, body: typeof autoMapped) {
  await page.route('**/api/imports', async (route: Route) => {
    if (route.request().method() === 'POST') await route.fulfill({ json: body });
    else await route.fallback();
  });
  await page.route(`**/api/imports/${body.batch_id}`, async (route: Route) => {
    if (route.request().method() === 'DELETE')
      await route.fulfill({ json: { id: body.batch_id, status: 'deleted' } });
    else
      await route.fulfill({
        json: {
          batch: {
            id: body.batch_id,
            filename: body.filename,
            status: 'parsed',
            currency: 'EUR',
            account_id: 1,
            movements_count: 2,
          },
          movements: body.valid,
          headers: body.headers,
          detected_delimiter: ',',
        },
      });
  });
  await page.route('**/api/imports/*/results', (r) => r.fulfill({ json: fixtures.results }));
}

async function stage(page: Page, name: string, content: string | Buffer) {
  const wizard = new ImportWizardPage(page);
  await page.goto('/imports');
  await wizard.openWizard();
  await wizard.selectAccount('Checking');
  await wizard.selectFile(csvPath(name, content));
  return wizard;
}

test.describe('Import > Wizard refresh', () => {
  test.beforeEach(async ({ mockImportsPage }) => {
    await mockImportsPage({ imports: [] });
  });

  test('N1 file card shows name and size; Change opens the picker; Remove returns to the dropzone', async ({
    page,
  }) => {
    await mockUpload(page, autoMapped);
    const wizard = await stage(page, 'bank.csv', CSV);
    const card = page.getByRole('group', { name: 'Selected file' });
    await expect(card).toContainText('bank.csv');
    await expect(card).toContainText(/KB/);
    const chooser = page.waitForEvent('filechooser');
    await card.getByRole('button', { name: 'Change file' }).click();
    await chooser;
    await card.getByRole('button', { name: 'Remove' }).click();
    await expect(card).toHaveCount(0);
    await expect(wizard.dropzone).toBeVisible();
    await expect(wizard.nextButton).toBeDisabled();
  });

  test('N2 preview: renamed headers, MoneyText amounts, invalid-rows card with raw line and Back to mapping', async ({
    page,
  }) => {
    await mockUpload(page, autoMapped);
    const wizard = await stage(page, 'bank.csv', CSV);
    await wizard.confirmUpload();
    for (const h of ['#', 'Date', 'Amount', 'Description']) {
      await expect(page.getByRole('columnheader', { name: h, exact: true })).toBeVisible();
    }
    await expect(page.getByRole('cell', { name: /10\.00/ }).first()).toBeVisible();
    const toggle = page.getByRole('button', { name: 'Invalid rows (1)' });
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');
    await expect(page.getByText('#4 · invalid amount')).toBeVisible();
    await expect(page.locator('code', { hasText: '2026-08-03,abc,Broken' })).toBeVisible();
    await page.getByRole('button', { name: 'Back to mapping' }).click();
    await expect(wizard.mappingDateSelect).toBeVisible();
    await expect(wizard.nextButton).toBeVisible();
  });

  test('N3 mapping shows sample values per picked column', async ({ page }) => {
    await mockUpload(page, unmapped);
    const wizard = await stage(page, 'unknown.csv', CSV_UNMAPPED);
    await wizard.confirmUpload();
    await expect(wizard.mappingDateSelect).toBeVisible({ timeout: 5000 });
    await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
    const lists = page.locator('ul[aria-label^="Sample values"]');
    await expect(lists).toHaveCount(3);
    await expect(lists.nth(0).getByRole('listitem').first()).toContainText(
      /05\/01\/2026\s*→\s*Jan 5, 2026/,
    );
    await expect(lists.nth(1).getByRole('listitem').first()).toHaveText('-10');
    await expect(lists.nth(2).getByRole('listitem').first()).toHaveText('Coffee');
  });

  // F7: mapping samples decode like the backend (UTF-16 BOM, strict UTF-8, windows-1252 fallback).
  const ROW_ES = ['Fecha,Valor,Desc', '2026-08-01,-10,Descripción', ''].join('\n');
  for (const [name, bytes] of [
    ['windows-1252', Buffer.from(ROW_ES, 'latin1')],
    [
      'UTF-16LE with BOM',
      Buffer.concat([Buffer.from([0xff, 0xfe]), Buffer.from(ROW_ES, 'utf16le')]),
    ],
    ['UTF-8', Buffer.from(ROW_ES, 'utf8')],
  ] as const) {
    test(`N8 mapping samples decode ${name}`, async ({ page }) => {
      await mockUpload(page, unmapped);
      const wizard = await stage(page, 'unknown.csv', bytes);
      await wizard.confirmUpload();
      await expect(wizard.mappingDateSelect).toBeVisible({ timeout: 5000 });
      await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
      const desc = page.locator('ul[aria-label^="Sample values"]').nth(2);
      await expect(desc.getByRole('listitem').first()).toHaveText('Descripción');
    });
  }

  // Amount samples are the raw cell (no client-side parsing); quoted fields come out unquoted.
  for (const [a, b, expA, expB] of [
    ['-18990.00', '"1.234,56"', '-18990.00', '1.234,56'],
    ['"1,234"', '"12,3,4"', '1,234', '12,3,4'],
  ] as const) {
    test(`N7 raw amount samples: ${a} / ${b}`, async ({ page }) => {
      await mockUpload(page, unmapped);
      const csv = `Fecha,Valor,Desc
2026-08-01,${a},A
2026-08-02,${b},B
`;
      const wizard = await stage(page, 'amounts.csv', csv);
      await wizard.confirmUpload();
      await expect(wizard.mappingDateSelect).toBeVisible({ timeout: 5000 });
      await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
      const items = page.locator('ul[aria-label^="Sample values"]').nth(1).getByRole('listitem');
      await expect(items.nth(0)).toHaveText(expA);
      await expect(items.nth(1)).toHaveText(expB);
    });
  }

  test('N8 a quote in the middle of a field does not swallow the amount column', async ({
    page,
  }) => {
    await mockUpload(page, { ...unmapped, headers: ['Fecha', 'Desc', 'Valor'] });
    const csv = 'Fecha,Desc,Valor\n2026-08-01,Pipe 3" wide,-25.50\n2026-08-02,Plain,-7.00\n';
    const wizard = await stage(page, 'inch.csv', csv);
    await wizard.confirmUpload();
    await expect(wizard.mappingDateSelect).toBeVisible({ timeout: 5000 });
    await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
    const lists = page.locator('ul[aria-label^="Sample values"]');
    await expect(lists.nth(1).getByRole('listitem').nth(0)).toHaveText('-25.50');
    await expect(lists.nth(1).getByRole('listitem').nth(1)).toHaveText('-7.00');
    await expect(lists.nth(2).getByRole('listitem').nth(0)).toHaveText('Pipe 3" wide');
  });

  test('N4 resume has no file in memory: fallback text instead of sample lists', async ({
    page,
  }) => {
    await page.unroute('**/api/imports*');
    const handler = async (route: Route) => {
      const url = route.request().url();
      const method = route.request().method();
      if (method === 'GET' && /\/api\/imports\/\d+$/.test(url)) {
        await route.fulfill({
          json: {
            batch: {
              id: 91,
              filename: 'resume.csv',
              status: 'parsed',
              currency: 'EUR',
              account_id: 1,
              movements_count: 2,
            },
            movements: autoMapped.valid,
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
          },
        });
      } else if (method === 'GET' && url.endsWith('/api/imports')) {
        await route.fulfill({
          json: [
            {
              id: 91,
              filename: 'resume.csv',
              file_hash: 'h91',
              format: 'csv',
              currency: 'EUR',
              status: 'parsed',
              created_at: '2026-08-21T10:00:00Z',
              account_id: 1,
              account_name: 'Checking',
              movements_count: 2,
            },
          ],
        });
      } else await route.fallback();
    };
    await page.route('**/api/imports', handler);
    await page.route('**/api/imports/**', handler);
    const wizard = new ImportWizardPage(page);
    await new ImportsPage(page).goto();
    await new ImportsPage(page)
      .getRowByFilename('resume.csv')
      .getByRole('button', { name: /Resume/ })
      .click();
    await expect(wizard.reconcileButton).toBeVisible({ timeout: 5000 });
    await wizard.backButton.click();
    await wizard.nextButton.click();
    await expect(wizard.mappingDateSelect).toBeVisible({ timeout: 5000 });
    await expect(page.getByText("Sample values aren't available when resuming")).toBeVisible();
    await expect(page.locator('ul[aria-label^="Sample values"]')).toHaveCount(0);
  });

  test('N5 tolerance control: group, default 0, pressed state, sent in the reconcile body', async ({
    page,
  }) => {
    await mockUpload(page, autoMapped);
    let tolerance: number | undefined;
    await page.route('**/api/imports/10/reconcile', async (route) => {
      tolerance = route.request().postDataJSON().tolerance_days;
      await route.fulfill({ json: { batch_id: 10, reconciliation_id: 1 } });
    });
    const wizard = await stage(page, 'bank.csv', CSV);
    await wizard.confirmUpload();
    await wizard.nextButton.click();
    const group = page.getByRole('group', { name: 'Tolerance' });
    await expect(group.getByRole('button', { name: '0', exact: true })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    await group.getByRole('button', { name: '±3' }).click();
    await expect(group.getByRole('button', { name: '±3' })).toHaveAttribute('aria-pressed', 'true');
    await expect(group.getByRole('button', { name: '0', exact: true })).toHaveAttribute(
      'aria-pressed',
      'false',
    );
    await wizard.reconcileButton.click();
    await expect(page).toHaveURL(/\/results\/10/);
    expect(tolerance).toBe(3);
  });

  test.describe('at 390px', () => {
    test.use({ viewport: { width: 390, height: 844 } });

    test('N6 every step fits: no horizontal overflow, dialog inside the viewport', async ({
      page,
    }) => {
      await mockUpload(page, autoMapped);
      const wizard = await stage(page, 'bank.csv', CSV);
      const check = async (what: string) => {
        const over = await page.evaluate(
          () => document.documentElement.scrollWidth - window.innerWidth,
        );
        expect(over, `${what}: page overflow`).toBeLessThanOrEqual(0);
        const d = await wizard.dialog.evaluate((el) => ({
          over: el.scrollWidth - el.clientWidth,
          r: el.getBoundingClientRect().right,
          h: el.getBoundingClientRect().height,
        }));
        expect(d.over, `${what}: dialog content overflow`).toBeLessThanOrEqual(0);
        expect(d.r, `${what}: dialog right edge`).toBeLessThanOrEqual(390);
        expect(d.h, `${what}: max 90dvh`).toBeLessThanOrEqual(844 * 0.9 + 1);
      };
      await check('upload');
      await wizard.confirmUpload();
      await expect(wizard.previewTable).toBeVisible();
      await check('preview + invalid rows');
      await wizard.nextButton.click();
      await expect(page.getByRole('group', { name: 'Tolerance' })).toBeVisible();
      await check('reconcile');
    });
  });
});
