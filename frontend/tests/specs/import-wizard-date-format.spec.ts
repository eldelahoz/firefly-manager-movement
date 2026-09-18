import { test, expect } from '../fixtures/api.fixture';
import { mockSettings } from '../mocks';
import { ImportWizardPage } from '../pages/ImportWizardPage';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

function csvPath(name: string, content: string): string {
  const p = path.join(os.tmpdir(), name);
  fs.writeFileSync(p, content);
  return p;
}

const csvAmbiguous = 'Fecha,Valor,Desc\n05/01/2026,-10,Coffee\n06/01/2026,-20,Book\n';

const probe = {
  batch_id: 100,
  filename: 'dates.csv',
  file_hash: 'h100',
  headers: ['Fecha', 'Valor', 'Desc'],
  detected_delimiter: ',',
  total_rows: 2,
  valid_count: 0,
  invalid_count: 0,
  mapping_used: null,
  origin_account_values: [],
  valid: [],
  invalid: [],
};

const formatSelect = /Date format|Formato de fecha/i;

test.describe('Import > Wizard date format (RF-02 v0.8)', () => {
  test.beforeEach(async ({ mockImportsPage, page }) => {
    await mockImportsPage({ imports: [] });
    await mockSettings(page);
  });

  async function openMappingStep(page: import('@playwright/test').Page, wizard: ImportWizardPage) {
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({ json: probe });
        return;
      }
      await route.fallback();
    });
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    await wizard.uploadFile(csvPath('dates.csv', csvAmbiguous));
    await expect(page.getByText(/We could not detect|No pudimos detectar/i)).toBeVisible({
      timeout: 5000,
    });
    await wizard.nextButton.click();
    await expect(wizard.mappingDateSelect).toBeVisible();
  }

  test('DF1 preview follows the selected format', async ({ page }) => {
    const wizard = new ImportWizardPage(page);
    await openMappingStep(page, wizard);
    await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
    const preview = page.getByTestId('date-format-preview');
    await expect(preview).toBeVisible();
    // Auto-detect: legacy order tries dd/mm/yyyy before mm/dd/yyyy
    await expect(preview).toContainText('2026-01-05');
    await page.getByRole('combobox', { name: formatSelect }).click();
    await page.getByRole('option', { name: 'MM/DD/YYYY' }).click();
    await expect(preview).toContainText('2026-05-01');
    await expect(preview).toContainText('2026-06-01');
    await page.getByRole('combobox', { name: formatSelect }).click();
    await page.getByRole('option', { name: 'DD/MM/YYYY' }).click();
    await expect(preview).toContainText('2026-01-05');
    await expect(preview).toContainText('2026-01-06');
  });

  test('DF2 ambiguous warning in auto mode', async ({ page }) => {
    const wizard = new ImportWizardPage(page);
    await openMappingStep(page, wizard);
    await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
    await expect(page.getByText(/Ambiguous dates|Fechas ambiguas/i)).toBeVisible({
      timeout: 5000,
    });
  });

  test('DF3 saved mapping carries the format; re-upload multipart and skip mapping', async ({
    page,
  }) => {
    const wizard = new ImportWizardPage(page);
    let putBody: { date_format?: string } | null = null;
    let multipart = '';
    let mapped = false;
    await page.route('**/api/imports/*/mapping', async (route) => {
      if (route.request().method() === 'PUT') {
        putBody = route.request().postDataJSON();
        await route.fulfill({ json: { mapping_id: 7 } });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/imports/100', async (route) => {
      if (route.request().method() === 'DELETE') {
        await route.fulfill({ json: { id: 100, status: 'deleted' } });
        return;
      }
      await route.fulfill({
        json: {
          batch: { id: 100, filename: 'dates.csv', status: 'parsed', currency: 'EUR', account_id: 1 },
          movements: [],
          headers: ['Fecha', 'Valor', 'Desc'],
          detected_delimiter: ',',
        },
      });
    });
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        multipart = route.request().postData() ?? '';
        if (!mapped) {
          await route.fulfill({ json: probe });
        } else {
          await route.fulfill({
            json: {
              ...probe,
              total_rows: 2,
              valid_count: 2,
              mapping_used: {
                date_column: 'Fecha',
                amount_column: 'Valor',
                description_column: 'Desc',
                date_format: 'mm/dd/yyyy',
              },
              valid: [
                { line_no: 2, date: '2026-05-01', amount_cents: -1000, description: 'Coffee' },
                { line_no: 3, date: '2026-06-01', amount_cents: -2000, description: 'Book' },
              ],
            },
          });
        }
        return;
      }
      await route.fallback();
    });
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    await wizard.uploadFile(csvPath('dates3.csv', csvAmbiguous));
    await expect(page.getByText(/We could not detect|No pudimos detectar/i)).toBeVisible({
      timeout: 5000,
    });
    await wizard.nextButton.click();
    await expect(wizard.mappingDateSelect).toBeVisible();
    await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
    await page.getByRole('combobox', { name: formatSelect }).click();
    await page.getByRole('option', { name: 'MM/DD/YYYY' }).click();
    mapped = true;
    await wizard.nextButton.click();
    await expect(page.getByRole('heading', { name: /Reconcile|Conciliar/ })).toBeVisible({
      timeout: 5000,
    });
    expect(putBody?.date_format).toBe('mm/dd/yyyy');
    expect(multipart).toContain('name="date_format"');
    expect(multipart).toContain('mm/dd/yyyy');
  });

  test('DF4 memorized signature with date_format skips mapping', async ({ page }) => {
    const wizard = new ImportWizardPage(page);
    await page.route('**/api/imports', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          json: {
            ...probe,
            total_rows: 2,
            valid_count: 2,
            mapping_used: {
              date_column: 'Fecha',
              amount_column: 'Valor',
              description_column: 'Desc',
              date_format: 'dd/mm/yyyy',
            },
            valid: [
              { line_no: 2, date: '2026-01-05', amount_cents: -1000, description: 'Coffee' },
              { line_no: 3, date: '2026-01-06', amount_cents: -2000, description: 'Book' },
            ],
          },
        });
        return;
      }
      await route.fallback();
    });
    await wizard.page.goto('/imports');
    await wizard.openWizard();
    await wizard.selectAccount('Checking');
    await wizard.uploadFile(csvPath('dates4.csv', csvAmbiguous));
    await expect(page.getByText(/auto-applied|aplicado automáticamente/i)).toBeVisible({
      timeout: 5000,
    });
    await wizard.nextButton.click();
    await expect(page.getByRole('heading', { name: /Reconcile|Conciliar/ })).toBeVisible({
      timeout: 5000,
    });
  });

  test('DF5 settings default pre-fills the format select', async ({ page }) => {
    await mockSettings(page, { settingsOverrides: { default_date_format: 'dd/mm/yyyy' } });
    const wizard = new ImportWizardPage(page);
    await openMappingStep(page, wizard);
    await wizard.selectMapping({ date: 'Fecha', amount: 'Valor', desc: 'Desc' });
    await expect(page.getByRole('combobox', { name: formatSelect })).toHaveText(/DD\/MM\/YYYY/, {
      timeout: 5000,
    });
  });
});
