import { test, expect } from '../fixtures/api.fixture';
import type { Route } from '@playwright/test';
import { ImportsPage } from '../pages/ImportsPage';
import { ImportWizardPage } from '../pages/ImportWizardPage';
import { fixtures } from '../fixtures/fixtures';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

function csvPath(name: string, content: string): string {
  const p = path.join(os.tmpdir(), name);
  fs.writeFileSync(p, content);
  return p;
}

const csvValid = 'date,amount,desc\n2026-08-01,-10,Coffee\n2026-08-02,-20,Book\n';
const csvNeedsMapping = 'Fecha,Valor,Desc\n2026-08-01,-10,Coffee\n';

test.describe('Import > Wizard', () => {
  test.beforeEach(async ({ mockImportsPage }) => {
    await mockImportsPage({ imports: [] });
  });

  test('W1 upload success auto-mapped', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 10,
            filename: 'bank.csv',
            file_hash: 'h10',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 2,
            valid_count: 2,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [
              { line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' },
              { line_no: 3, date: '2026-08-02', amount_cents: -2000, description: 'Book' },
            ],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/*/results', async (route) => route.fulfill({ json: fixtures.results }));
    await page.route('**/api/firefly/accounts*', async (route) => {
      if (route.request().url().includes('/transactions')) await route.fallback();
      else await route.fulfill({ json: fixtures.accounts });
    });

    const importsPage = new ImportsPage(page);
    const wizard = new ImportWizardPage(page);
    await importsPage.goto();
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('bank.csv', csvValid);
    await wizard.uploadFile(p);
    await expect(page.getByText(/auto-applied/i)).toBeVisible({ timeout: 5000 });
    await wizard.nextButton.click();
    // Should skip mapping and go to reconcile
    await expect(page.getByRole('heading', { name: /Reconcile|Conciliar/ })).toBeVisible({ timeout: 5000 });
  });

  test('W19 upload requires explicit Next confirmation', async ({ page }) => {
    let postCount = 0;
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        postCount += 1;
        await route.fulfill({
          json: {
            batch_id: 10,
            filename: 'confirm.csv',
            file_hash: 'h10',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 2,
            valid_count: 2,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [
              { line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' },
              { line_no: 3, date: '2026-08-02', amount_cents: -2000, description: 'Book' },
            ],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });

    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    // Next disabled until a file is staged
    await expect(wizard.nextButton).toBeDisabled();
    await expect(page.getByText(/Select a CSV file|Selecciona un archivo CSV/)).toBeVisible();

    const p = csvPath('confirm.csv', csvValid);
    await wizard.selectFile(p);
    // Wizard stays on step 1 with the staged file; no request sent
    await expect(page.getByText('confirm.csv')).toBeVisible();
    await expect(wizard.nextButton).toBeEnabled();
    expect(postCount).toBe(0);

    await wizard.confirmUpload();
    await expect(page.getByText(/auto-applied/i)).toBeVisible({ timeout: 5000 });
    expect(postCount).toBe(1);
  });

  test('W20 remove discards parsed probe batch', async ({ page }) => {
    let deleteCount = 0;
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 50,
            filename: 'bank.csv',
            file_hash: 'h50',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/50', async (route) => {
      if (route.request().method() === 'DELETE') {
        deleteCount += 1;
        await route.fulfill({ json: { id: 50, status: 'deleted' } });
        return;
      }
      await route.fallback();
    });

    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('bank.csv', csvValid);

    // Staged but never parsed: Remove is purely local, no DELETE
    await wizard.selectFile(p);
    await page.getByRole('button', { name: /Remove|Eliminar|Retirer/ }).click();
    await expect(wizard.dropzone).toBeVisible();
    expect(deleteCount).toBe(0);

    // Parsed, then back (preview -> mapping -> upload), then remove: the probe batch is deleted server-side
    await wizard.uploadFile(p);
    await expect(page.getByText(/auto-applied/i)).toBeVisible({ timeout: 5000 });
    await wizard.backButton.click();
    await wizard.backButton.click();
    await page.getByRole('button', { name: /Remove|Eliminar|Retirer/ }).click();
    await expect(wizard.dropzone).toBeVisible();
    expect(deleteCount).toBe(1);
    await expect(wizard.nextButton).toBeDisabled();
  });

  test('W2 upload needs mapping', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 11,
            filename: 'unknown.csv',
            file_hash: 'h11',
            headers: ['Fecha', 'Valor', 'Desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 0,
            invalid_count: 0,
            mapping_used: null,
            origin_account_values: [],
            valid: [],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/*/mapping', async (route) => {
      if (route.request().method() === 'PUT') {
        await route.fulfill({ json: { mapping_id: 1 } });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/11', async (route) => {
      if (route.request().method() === 'DELETE') {
        await route.fulfill({ json: { id: 11, status: 'deleted' } });
        return;
      }
      await route.fulfill({ json: { batch: { id: 11, filename: 'unknown.csv', status: 'parsed', currency: 'EUR', account_id: 1, movements_count: 0 }, movements: [], headers: ['Fecha', 'Valor', 'Desc'], detected_delimiter: ';' } });
    });

    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('unknown.csv', csvNeedsMapping);
    await wizard.uploadFile(p);
    // No memorized mapping: the wizard lands on mapping with the detection-failure hint
    await expect(page.getByText(/We could not detect|No pudimos detectar/)).toBeVisible({ timeout: 5000 });
    await expect(wizard.mappingDateSelect).toBeVisible();
    await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
    await expect(wizard.nextButton).toBeEnabled();
    // Mock re-upload after mapping
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 11,
            filename: 'unknown.csv',
            file_hash: 'h11',
            headers: ['Fecha', 'Valor', 'Desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'Fecha', amount_column: 'Valor', description_column: 'Desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await wizard.nextButton.click();
    // Preview shows the rows parsed with the chosen mapping (manual: no auto-applied badge)
    await expect(wizard.previewTable).toBeVisible();
    await expect(page.getByRole('cell', { name: 'Coffee' })).toBeVisible();
    await expect(page.getByText(/auto-applied/i)).toBeHidden();
    await wizard.nextButton.click();
    await expect(page.getByRole('heading', { name: /Reconcile|Conciliar/ })).toBeVisible({ timeout: 5000 });
  });

  test('W3 drag-drop', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 12,
            filename: 'drag.csv',
            file_hash: 'h12',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('drag.csv', csvValid);
    // Test drag-drop via DataTransfer
    const buffer = fs.readFileSync(p);
    await wizard.page.locator('input[type="file"]').setInputFiles({ name: 'drag.csv', mimeType: 'text/csv', buffer });
    await expect(page.getByText('drag.csv')).toBeVisible();
    await wizard.confirmUpload();
    await expect(page.getByText(/auto-applied|aplicado automáticamente/i)).toBeVisible({ timeout: 5000 });
  });

  test('W3b drag-drop animation', async ({ page }) => {
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const dropzone = page.locator('[role="button"]').first();
    await dropzone.dispatchEvent('dragover', { dataTransfer: await page.evaluateHandle(() => new DataTransfer()) });
    await expect(page.getByText(/Drop to upload|Suelta para subir/)).toBeVisible();
    await expect(dropzone).toHaveClass(/border-accent/);
    await dropzone.dispatchEvent('dragleave');
    await expect(page.getByText(/Drag CSV|Arrastra/)).toBeVisible();
  });

  test('W4 duplicate filename case-insensitive', async ({ page }) => {
    await page.route('**/api/imports*', async (route) => {
      if (route.request().method() === 'GET' && !route.request().url().includes('/results')) {
        await route.fulfill({ json: [{ id: 1, filename: 'bank.csv', file_hash: 'h1', format: 'csv', currency: 'EUR', status: 'parsed', created_at: '2026-08-20T10:00:00Z', account_id: 1, account_name: 'Checking', movements_count: 2 }] });
        return;
      }
      await route.fallback();
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('BANK.csv', csvValid);
    await wizard.selectFile(p);
    await expect(page.getByText(/already exists|Ya existe/)).toBeVisible({ timeout: 5000 });
    // Duplicate is refused at staging: nothing was staged, Next stays disabled
    await expect(wizard.nextButton).toBeDisabled();
  });

  test('W5 oversized 10MB', async ({ page }) => {
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    // Create a file with size >10MB via buffer
    const bigBuffer = Buffer.alloc(11 * 1024 * 1024, 'a');
    await wizard.page.locator('input[type="file"]').setInputFiles({ name: 'big.csv', mimeType: 'text/csv', buffer: bigBuffer });
    await expect(page.getByText(/too large|demasiado grande/)).toBeVisible({ timeout: 5000 });
  });

  test('W6 reconcile with tolerance', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 20,
            filename: 'bank.csv',
            file_hash: 'h20',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 2,
            valid_count: 2,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [
              { line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' },
              { line_no: 3, date: '2026-08-02', amount_cents: -2000, description: 'Book' },
            ],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/20', async (route) => route.fulfill({ json: { batch: { id: 20, filename: 'bank.csv', status: 'parsed', currency: 'EUR', account_id: 1 }, movements: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }], headers: ['date', 'amount', 'desc'], detected_delimiter: ',' } }));
    await page.route('**/api/imports/20/reconcile', async (route) => {
      const body = route.request().postDataJSON();
      expect(body.tolerance_days).toBe(2);
      expect(body.window_start).toBe('2026-07-30');
      expect(body.window_end).toBe('2026-08-03');
      await route.fulfill({ json: { batch_id: 20, reconciliation_id: 1 } });
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('bank.csv', csvValid);
    await wizard.uploadFile(p);
    await wizard.nextButton.click();
    // In reconcile step, select tolerance 2
    await page.getByRole('button', { name: '±2' }).click();
    await wizard.reconcileButton.click();
    await expect(page).toHaveURL(/\/results\/20/);
  });

  test('W24 search window derived from tolerance and read-only', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 22,
            filename: 'bank.csv',
            file_hash: 'h22',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/22', async (route) => route.fulfill({ json: { batch: { id: 22, filename: 'bank.csv', status: 'parsed', currency: 'EUR', account_id: 1 }, movements: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }], headers: ['date', 'amount', 'desc'], detected_delimiter: ',' } }));
    await page.route('**/api/imports/22/reconcile', async (route) => {
      const body = route.request().postDataJSON();
      expect(body.tolerance_days).toBe(2);
      expect(body.window_start).toBe('2026-07-30');
      expect(body.window_end).toBe('2026-08-03');
      await route.fulfill({ json: { batch_id: 22, reconciliation_id: 1 } });
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('bank.csv', csvValid);
    await wizard.uploadFile(p);
    await wizard.nextButton.click();
    await expect(page.getByText('Search window')).toBeVisible();
    await expect(page.getByText('2026-08-01 → 2026-08-01')).toBeVisible();
    await page.getByRole('button', { name: '±2' }).click();
    await expect(page.getByText('2026-07-30 → 2026-08-03')).toBeVisible();
    await expect(page.locator('.fixed input[type="date"], [role="dialog"] input[type="date"]')).toHaveCount(0);
    await wizard.reconcileButton.click();
    await expect(page).toHaveURL(/\/results\/22/);
  });

  test('W25 search window manual override with auto reset', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 25,
            filename: 'bank.csv',
            file_hash: 'h25',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/25', async (route) => route.fulfill({ json: { batch: { id: 25, filename: 'bank.csv', status: 'parsed', currency: 'EUR', account_id: 1 }, movements: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }], headers: ['date', 'amount', 'desc'], detected_delimiter: ',' } }));
    await page.route('**/api/imports/25/reconcile', async (route) => {
      const body = route.request().postDataJSON();
      expect(body.tolerance_days).toBe(0);
      expect(body.window_start).toBe('2026-07-01');
      expect(body.window_end).toBe('2026-08-01');
      await route.fulfill({ json: { batch_id: 25, reconciliation_id: 1 } });
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('bank.csv', csvValid);
    await wizard.uploadFile(p);
    await wizard.nextButton.click();
    const dialog = page.getByRole('dialog');
    await expect(dialog.getByText('2026-08-01 → 2026-08-01')).toBeVisible();
    await dialog.getByRole('button', { name: 'Edit' }).click();
    const startInput = dialog.locator('input[type="date"]').first();
    const endInput = dialog.locator('input[type="date"]').nth(1);
    await expect(startInput).toHaveValue('2026-08-01');
    await expect(endInput).toHaveValue('2026-08-01');
    await startInput.fill('2026-07-01');
    await dialog.getByRole('button', { name: 'Auto' }).click();
    await expect(dialog.getByText('2026-08-01 → 2026-08-01')).toBeVisible();
    await expect(dialog.locator('input[type="date"]')).toHaveCount(0);
    await dialog.getByRole('button', { name: 'Edit' }).click();
    await dialog.locator('input[type="date"]').first().fill('2026-07-01');
    await wizard.reconcileButton.click();
    await expect(page).toHaveURL(/\/results\/25/);
  });

  test('W7 currency mismatch', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 21,
            filename: 'bank.csv',
            file_hash: 'h21',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/21', async (route) => route.fulfill({ json: { batch: { id: 21, filename: 'bank.csv', status: 'parsed', currency: 'USD', account_id: null }, movements: [], headers: ['date', 'amount', 'desc'], detected_delimiter: ',' } }));
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    // Select USD account but batch is USD, then change to EUR to mismatch? Use Checking (EUR) vs batch USD
    await wizard.selectAccount('Checking'); // EUR
    const p = csvPath('bank.csv', csvValid);
    await wizard.uploadFile(p);
    await wizard.nextButton.click();
    // Now in reconcile, batch currency USD vs account EUR -> mismatch
    await expect(page.getByText(/mismatch|Currency mismatch/).first()).toBeVisible();
    await expect(wizard.reconcileButton).toBeDisabled();
  });

  test('W8 cancel from upload', async ({ page }) => {
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.cancelButton.click();
    await expect(wizard.dialog).toBeHidden();
  });

  test('W9 cancel from preview', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 30,
            filename: 'bank.csv',
            file_hash: 'h30',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('bank.csv', csvValid);
    await wizard.uploadFile(p);
    await expect(page.getByText(/auto-applied/i)).toBeVisible({ timeout: 5000 });
    await wizard.cancelButton.click();
    await expect(wizard.dialog).toBeHidden();
  });

  test('W10 cancel from mapping', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 31,
            filename: 'unknown.csv',
            file_hash: 'h31',
            headers: ['Fecha', 'Valor', 'Desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 0,
            invalid_count: 0,
            mapping_used: null,
            origin_account_values: [],
            valid: [],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('unknown.csv', csvNeedsMapping);
    await wizard.uploadFile(p);
    // Unmapped upload lands directly on the mapping step
    await expect(wizard.mappingDateSelect).toBeVisible();
    await wizard.cancelButton.click();
    await expect(wizard.dialog).toBeHidden();
  });

  test('W23 fresh reconcile shows the import account locked', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 41,
            filename: 'bank.csv',
            file_hash: 'h41',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/41', async (route) => route.fulfill({ json: { batch: { id: 41, filename: 'bank.csv', status: 'parsed', currency: 'EUR', account_id: 1 }, movements: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }], headers: ['date', 'amount', 'desc'], detected_delimiter: ',' } }));
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    await wizard.uploadFile(csvPath('bank.csv', csvValid));
    await wizard.nextButton.click();
    await expect(wizard.reconcileButton).toBeVisible();
    // Locked (RF-04 v0.8): account name + hint pill, no account combobox at all
    await expect(page.getByText('Checking').first()).toBeVisible();
    await expect(page.getByText(/selected during import|seleccionada durante la importación/i)).toBeVisible();
    await expect(page.getByRole('combobox', { name: /Firefly III account|Cuenta de Firefly/ })).toHaveCount(0);
  });

  test('W11 cancel from reconcile', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 40,
            filename: 'bank.csv',
            file_hash: 'h40',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/40', async (route) => route.fulfill({ json: { batch: { id: 40, filename: 'bank.csv', status: 'parsed', currency: 'EUR', account_id: 1 }, movements: [], headers: ['date', 'amount', 'desc'], detected_delimiter: ',' } }));
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('bank.csv', csvValid);
    await wizard.uploadFile(p);
    await wizard.nextButton.click();
    await expect(wizard.reconcileButton).toBeVisible();
    await wizard.cancelButton.click();
    await expect(wizard.dialog).toBeHidden();
  });

  test('W12 resume parsed>0', async ({ page }) => {
    await page.unroute('**/api/imports*');
    const importsHandler = async (route: Route) => {
      const url = route.request().url();
      if (route.request().method() === 'GET' && url.match(/\/api\/imports\/\d+$/)) {
        await route.fulfill({ json: { batch: { id: 1, filename: 'bank.csv', status: 'parsed', currency: 'EUR', account_id: 1, movements_count: 2 }, movements: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }], headers: ['date', 'amount', 'desc'], detected_delimiter: ',' } });
        return;
      }
      if (route.request().method() === 'GET' && url.endsWith('/api/imports')) {
        await route.fulfill({ json: [{ id: 1, filename: 'bank.csv', file_hash: 'h1', format: 'csv', currency: 'EUR', status: 'parsed', created_at: '2026-08-20T10:00:00Z', account_id: 1, account_name: 'Checking', movements_count: 2 }] });
        return;
      }
      await route.fallback();
    };
    await page.route('**/api/imports', importsHandler);
    await page.route('**/api/imports/**', importsHandler);
    const wizard = new ImportWizardPage(page);
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await importsPage.getRowByFilename('bank.csv').getByRole('button', { name: /Resume|Reanudar/ }).click();
    // Resume lands on reconcile-config with account locked (accountLockedHint)
    await expect(page.getByRole('heading', { name: /Reconcile|Conciliar/ })).toBeVisible({ timeout: 5000 });
    await expect(wizard.reconcileButton).toBeVisible({ timeout: 5000 });
    await expect(page.getByText(/selected during import|seleccionada durante la importación/i)).toBeVisible();
  });

  test('W13 resume parsed 0', async ({ page }) => {
    await page.unroute('**/api/imports*');
    await page.route('**/api/imports*', async (route) => {
      const url = route.request().url();
      if (route.request().method() === 'GET' && url.match(/\/api\/imports\/\d+$/)) {
        await route.fulfill({ json: { batch: { id: 2, filename: 'empty.csv', status: 'parsed', currency: 'EUR', account_id: null, movements_count: 0 }, movements: [], headers: ['Fecha', 'Valor', 'Desc'], detected_delimiter: ';' } });
        return;
      }
      if (route.request().method() === 'GET' && url.endsWith('/api/imports')) {
        await route.fulfill({ json: [{ id: 2, filename: 'empty.csv', file_hash: 'h2', format: 'csv', currency: 'EUR', status: 'parsed', created_at: '2026-08-11T10:00:00Z', account_id: null, account_name: null, movements_count: 0 }] });
        return;
      }
      await route.fallback();
    });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    // empty.csv has movements_count 0, so Resume should not be visible, only Delete
    await expect(importsPage.getRowByFilename('empty.csv').getByRole('button', { name: /Resume/ })).toBeHidden();
    await expect(importsPage.getRowByFilename('empty.csv').getByRole('button', { name: /Delete/ })).toBeVisible();
  });

  test('W14 resume after refresh', async ({ page }) => {
    await page.unroute('**/api/imports*');
    const handler = async (route: Route) => {
      const url = route.request().url();
      if (route.request().method() === 'GET' && url.match(/\/api\/imports\/\d+$/)) {
        await route.fulfill({ json: { batch: { id: 1, filename: 'bank.csv', status: 'parsed', currency: 'EUR', account_id: 1, movements_count: 2 }, movements: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }], headers: ['date', 'amount', 'desc'], detected_delimiter: ',' } });
        return;
      }
      if (route.request().method() === 'GET' && url.endsWith('/api/imports')) {
        await route.fulfill({ json: [{ id: 1, filename: 'bank.csv', file_hash: 'h1', format: 'csv', currency: 'EUR', status: 'parsed', created_at: '2026-08-20T10:00:00Z', account_id: 1, account_name: 'Checking', movements_count: 2 }] });
        return;
      }
      await route.fallback();
    };
    await page.route('**/api/imports', handler);
    await page.route('**/api/imports/**', handler);
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await importsPage.getRowByFilename('bank.csv').getByRole('button', { name: /Resume/ }).click();
    await expect(page.getByRole('dialog')).toBeVisible({ timeout: 5000 });
    // Resume state is not persisted: after reload the wizard is closed
    await page.reload();
    await expect(page.getByRole('dialog')).toBeHidden({ timeout: 5000 });
    // User returns to the list and resumes again
    await importsPage.getRowByFilename('bank.csv').getByRole('button', { name: /Resume/ }).click();
    await expect(page.getByRole('dialog')).toBeVisible({ timeout: 5000 });
  });

  test('W15 back/next upload<->preview', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 50,
            filename: 'bank.csv',
            file_hash: 'h50',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: [],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/50', async (route) => {
      if (route.request().method() === 'DELETE') {
        await route.fulfill({ json: { id: 50, status: 'deleted' } });
        return;
      }
      await route.fallback();
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('bank.csv', csvValid);
    await wizard.uploadFile(p);
    await expect(page.getByText(/auto-applied/i)).toBeVisible({ timeout: 5000 });
    await wizard.backButton.click();
    // Back navigates one step at a time: preview -> mapping (auto-applied mapping preselected)
    await expect(wizard.mappingDateSelect).toBeVisible();
    await expect(wizard.mappingDateSelect).toHaveText(/date/);
    await wizard.backButton.click();
    // File preserved: file panel shown, dropzone hidden, Next re-enabled
    await expect(page.getByText('bank.csv')).toBeVisible();
    await expect(wizard.dropzone).toBeHidden();
    await expect(wizard.nextButton).toBeEnabled();
    // Next re-parses (previous probe batch deleted first)
    await wizard.confirmUpload();
    await expect(page.getByText(/auto-applied/i)).toBeVisible({ timeout: 5000 });
    // Change file: back through mapping to upload, stage again, confirm re-parses
    await wizard.backButton.click();
    await wizard.backButton.click();
    const [fc] = await Promise.all([
      page.waitForEvent('filechooser'),
      page.getByRole('button', { name: /Change file|Cambiar archivo/ }).click(),
    ]);
    await fc.setFiles(p);
    await expect(page.getByText('bank.csv')).toBeVisible();
    await wizard.confirmUpload();
    await expect(page.getByText(/auto-applied/i)).toBeVisible({ timeout: 5000 });
  });

  test('W16 back/next upload<->mapping<->preview<->reconcile', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        const withColumns = (route.request().postData() ?? '').includes('date_column');
        await route.fulfill({
          json: {
            batch_id: 60,
            filename: 'unknown.csv',
            file_hash: 'h60',
            headers: ['Fecha', 'Valor', 'Desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: withColumns ? 1 : 0,
            invalid_count: 0,
            mapping_used: withColumns
              ? { date_column: 'Fecha', amount_column: 'Valor', description_column: 'Desc' }
              : null,
            origin_account_values: [],
            valid: withColumns
              ? [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }]
              : [],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/*/mapping', async (route) => route.fulfill({ json: { mapping_id: 1 } }));
    // Probe batch 60: detail for reconcile step + DELETE when mapping is saved
    await page.route('**/api/imports/60', async (route) => {
      if (route.request().method() === 'DELETE') {
        await route.fulfill({ json: { id: 60, status: 'deleted' } });
        return;
      }
      await route.fulfill({ json: { batch: { id: 60, filename: 'unknown.csv', status: 'parsed', currency: 'EUR', account_id: 1, movements_count: 0 }, movements: [], headers: ['Fecha', 'Valor', 'Desc'], detected_delimiter: ';' } });
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('unknown.csv', csvNeedsMapping);
    await wizard.uploadFile(p);
    // Upload without a memorized mapping lands directly on mapping
    await expect(page.getByText(/We could not detect|No pudimos detectar/i)).toBeVisible({ timeout: 5000 });
    await expect(wizard.mappingDateSelect).toBeVisible();
    await wizard.backButton.click();
    // Back from mapping returns to upload with the file still staged
    await expect(page.getByText('unknown.csv')).toBeVisible();
    await expect(wizard.dropzone).toBeHidden();
    await wizard.confirmUpload();
    await expect(wizard.mappingDateSelect).toBeVisible();
    await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
    await wizard.nextButton.click();
    // Preview: rows parsed with the chosen mapping (manual, so no auto-applied badge)
    await expect(wizard.previewTable).toBeVisible();
    await expect(page.getByRole('cell', { name: 'Coffee' })).toBeVisible();
    await expect(page.getByText(/auto-applied/i)).toBeHidden();
    await wizard.nextButton.click();
    await expect(wizard.reconcileButton).toBeVisible();
    // Back navigates preview -> mapping (selection preserved) -> upload
    await wizard.backButton.click();
    await expect(wizard.previewTable).toBeVisible();
    await wizard.backButton.click();
    await expect(wizard.mappingDateSelect).toBeVisible();
    await expect(wizard.mappingDateSelect).toHaveText(/Fecha/);
    await wizard.backButton.click();
    await expect(page.getByText('unknown.csv')).toBeVisible();
  });

  test('W17 mapping incomplete Next disabled', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 70,
            filename: 'unknown.csv',
            file_hash: 'h70',
            headers: ['Fecha', 'Valor', 'Desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 0,
            invalid_count: 0,
            mapping_used: null,
            origin_account_values: [],
            valid: [],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('unknown.csv', csvNeedsMapping);
    await wizard.uploadFile(p);
    await expect(wizard.mappingDateSelect).toBeVisible();
    await expect(wizard.nextButton).toBeDisabled();
    await wizard.mappingDateSelect.click();
    await page.getByRole('option', { name: 'Fecha' }).click();
    await expect(wizard.nextButton).toBeDisabled();
    await wizard.mappingAmountSelect.click();
    await page.getByRole('option', { name: 'Valor' }).click();
    await expect(wizard.nextButton).toBeDisabled();
    await wizard.mappingDescSelect.click();
    await page.getByRole('option', { name: 'Desc' }).click();
    await expect(wizard.nextButton).toBeEnabled();
  });

  test('W18 origin suggest auto', async ({ page }) => {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            batch_id: 80,
            filename: 'bank.csv',
            file_hash: 'h80',
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
            total_rows: 1,
            valid_count: 1,
            invalid_count: 0,
            mapping_used: { date_column: 'date', amount_column: 'amount', description_column: 'desc' },
            origin_account_values: ['Checking'],
            valid: [{ line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' }],
            invalid: [],
          },
        });
        return;
      }
      await route.fallback();
    });
    const wizard = new ImportWizardPage(page);
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    const p = csvPath('bank.csv', csvValid);
    await wizard.uploadFile(p);
    await expect(page.getByText(/auto-selected|seleccionada automáticamente/)).toBeVisible({ timeout: 5000 });
  });

  test('W2 resume allows mapping via Back then Next', async ({ page }) => {
    await page.unroute('**/api/imports*');
    const handler = async (route: Route) => {
      const url = route.request().url();
      if (route.request().method() === 'GET' && url.match(/\/api\/imports\/\d+$/)) {
        await route.fulfill({
          json: {
            batch: { id: 91, filename: 'resume.csv', status: 'parsed', currency: 'EUR', account_id: 1, movements_count: 2 },
            movements: [
              { line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' },
              { line_no: 3, date: '2026-08-02', amount_cents: -2000, description: 'Tea' },
            ],
            headers: ['date', 'amount', 'desc'],
            detected_delimiter: ',',
          },
        });
        return;
      }
      if (route.request().method() === 'GET' && url.endsWith('/api/imports')) {
        await route.fulfill({ json: [{ id: 91, filename: 'resume.csv', file_hash: 'h91', format: 'csv', currency: 'EUR', status: 'parsed', created_at: '2026-08-21T10:00:00Z', account_id: 1, account_name: 'Checking', movements_count: 2 }] });
        return;
      }
      if (route.request().method() === 'PUT' && url.includes('/mapping')) {
        await route.fulfill({ json: { mapping_id: 1 } });
        return;
      }
      await route.fallback();
    };
    await page.route('**/api/imports', handler);
    await page.route('**/api/imports/**', handler);
    const wizard = new ImportWizardPage(page);
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    // Resume lands on reconcile-config (state.ts RESUME_INIT)
    await importsPage.getRowByFilename('resume.csv').getByRole('button', { name: /Resume|Reanudar/ }).click();
    await expect(wizard.reconcileButton).toBeVisible({ timeout: 5000 });
    // Back -> preview
    await wizard.backButton.click();
    await expect(wizard.previewTable).toBeVisible();
    // Next -> mapping still reachable (not skipped)
    await wizard.nextButton.click();
    await expect(wizard.mappingDateSelect).toBeVisible({ timeout: 5000 });
  });

  test('W22 resume mapping shows persisted columns', async ({ page }) => {
    await page.unroute('**/api/imports*');
    const handler = async (route: Route) => {
      const url = route.request().url();
      if (route.request().method() === 'GET' && url.match(/\/api\/imports\/\d+$/)) {
        await route.fulfill({
          json: {
            batch: { id: 92, filename: 'unknown.csv', status: 'parsed', currency: 'EUR', account_id: 1, movements_count: 2 },
            movements: [
              { line_no: 2, date: '2026-08-01', amount_cents: -1000, description: 'Coffee' },
              { line_no: 3, date: '2026-08-02', amount_cents: -2000, description: 'Tea' },
            ],
            headers: ['Fecha', 'Valor', 'Desc'],
            detected_delimiter: ';',
          },
        });
        return;
      }
      if (route.request().method() === 'GET' && url.endsWith('/api/imports')) {
        await route.fulfill({ json: [{ id: 92, filename: 'unknown.csv', file_hash: 'h92', format: 'csv', currency: 'EUR', status: 'parsed', created_at: '2026-08-21T10:00:00Z', account_id: 1, account_name: 'Checking', movements_count: 2 }] });
        return;
      }
      await route.fallback();
    };
    await page.route('**/api/imports', handler);
    await page.route('**/api/imports/**', handler);
    const wizard = new ImportWizardPage(page);
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await importsPage.getRowByFilename('unknown.csv').getByRole('button', { name: /Resume|Reanudar/ }).click();
    await expect(wizard.reconcileButton).toBeVisible({ timeout: 5000 });
    // Back -> preview -> Next -> mapping must list the persisted CSV columns
    await wizard.backButton.click();
    await wizard.nextButton.click();
    await expect(wizard.mappingDateSelect).toBeVisible({ timeout: 5000 });
    await wizard.mappingDateSelect.click();
    await expect(page.getByRole('option', { name: 'Fecha' })).toBeVisible();
    await expect(page.getByRole('option', { name: 'Valor' })).toBeVisible();
    await expect(page.getByRole('option', { name: 'Desc' })).toBeVisible();
  });
});
