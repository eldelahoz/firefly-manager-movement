import { test, expect } from '../fixtures/api.fixture';
import { ImportsPage } from '../pages/ImportsPage';
import { fixtures } from '../fixtures/fixtures';

const baseImports = [
  {
    id: 1,
    filename: 'bank.csv',
    file_hash: 'h1',
    format: 'csv',
    currency: 'EUR',
    status: 'parsed' as const,
    created_at: '2026-08-10T10:00:00Z',
    account_id: 1,
    account_name: 'Checking',
    movements_count: 2,
  },
  {
    id: 2,
    filename: 'empty.csv',
    file_hash: 'h2',
    format: 'csv',
    currency: 'EUR',
    status: 'parsed' as const,
    created_at: '2026-08-11T10:00:00Z',
    account_id: null,
    account_name: null,
    movements_count: 0,
  },
  {
    id: 3,
    filename: 'recon.csv',
    file_hash: 'h3',
    format: 'csv',
    currency: 'EUR',
    status: 'reconciled' as const,
    created_at: '2026-08-15T10:00:00Z',
    account_id: 1,
    account_name: 'Checking',
    movements_count: 5,
  },
  {
    id: 4,
    filename: 'applied.csv',
    file_hash: 'h4',
    format: 'csv',
    currency: 'EUR',
    status: 'applied' as const,
    created_at: '2026-08-20T10:00:00Z',
    account_id: 1,
    account_name: 'Checking',
    movements_count: 3,
  },
];

test.describe('Imports > Page', () => {
  test('I1 renders table with mixed statuses', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await expect(importsPage.table).toBeVisible();
    await expect(importsPage.rows).toHaveCount(4);
    await expect(importsPage.getRowByFilename('bank.csv')).toBeVisible();
    await expect(importsPage.getRowByFilename('bank.csv').locator('text=Parsed').first()).toBeVisible();
    await expect(importsPage.getRowByFilename('recon.csv').locator('text=Reconciled').first()).toBeVisible();
  });

  test('I2 search filters by filename', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await expect(importsPage.rows).toHaveCount(4);
    await importsPage.search('bank');
    await expect(importsPage.rows).toHaveCount(1);
    await expect(importsPage.getRowByFilename('bank.csv')).toBeVisible();
    await importsPage.search('xxx');
    await expect(importsPage.emptyState).toBeVisible();
    await importsPage.search('');
    await expect(importsPage.rows).toHaveCount(4);
  });

  test('I3 status filter', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await importsPage.filterStatus('Parsed');
    await expect(importsPage.rows).toHaveCount(2);
    await importsPage.filterStatus('Reconciled');
    await expect(importsPage.rows).toHaveCount(1);
    await expect(importsPage.getRowByFilename('recon.csv')).toBeVisible();
    await importsPage.filterStatus('All');
    await expect(importsPage.rows).toHaveCount(4);
  });

  test('I4 date filters', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await importsPage.setFrom('2026-08-15');
    await expect(importsPage.rows).toHaveCount(2); // recon + applied (15,20)
    await importsPage.setTo('2026-08-15');
    await expect(importsPage.rows).toHaveCount(1); // only recon 15
    await importsPage.setFrom('');
    await importsPage.setTo('');
    await expect(importsPage.rows).toHaveCount(4);
    // max/min attrs
    await importsPage.setFrom('2026-08-20');
    await importsPage.setTo('2026-08-10');
    await expect(importsPage.emptyState).toBeVisible();
  });

  test('I5 combined filters', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await importsPage.search('bank');
    await importsPage.filterStatus('Parsed');
    await expect(importsPage.rows).toHaveCount(1);
    await expect(importsPage.getRowByFilename('bank.csv')).toBeVisible();
  });

  test('I6 resume shows for parsed>0', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await expect(importsPage.getActionButton('bank.csv', 'resume')).toBeVisible();
    await expect(importsPage.getActionButton('empty.csv', 'resume')).toBeHidden();
    await expect(importsPage.getActionButton('recon.csv', 'resume')).toBeHidden();
  });

  test('I7 viewResults + rerun for reconciled/applied', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await expect(importsPage.getActionButton('recon.csv', 'view')).toBeVisible();
    await expect(importsPage.getActionButton('recon.csv', 'rerun')).toBeVisible();
    await expect(importsPage.getActionButton('applied.csv', 'view')).toBeVisible();
    await expect(importsPage.getActionButton('bank.csv', 'view')).toBeHidden();
    await expect(importsPage.getActionButton('bank.csv', 'rerun')).toBeHidden();
  });

  test('I8 delete only for parsed', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await expect(importsPage.getActionButton('bank.csv', 'delete')).toBeVisible();
    await expect(importsPage.getActionButton('empty.csv', 'delete')).toBeVisible();
    await expect(importsPage.getActionButton('recon.csv', 'delete')).toBeHidden();
    await expect(importsPage.getActionButton('applied.csv', 'delete')).toBeHidden();
  });

  test('I9 delete dialog confirms and deletes', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await importsPage.getActionButton('bank.csv', 'delete').click();
    await expect(importsPage.deleteDialog).toBeVisible();
    await expect(importsPage.deleteDialog).toContainText('bank.csv');
    await expect(importsPage.deleteDialog).toContainText('2 movements');
    const deletePromise = page.waitForResponse(
      (r) => r.url().includes('/api/imports/1') && r.request().method() === 'DELETE',
    );
    await importsPage.deleteConfirmButton.click();
    await deletePromise;
    await expect(importsPage.deleteDialog).toBeHidden({ timeout: 5000 });
  });

  test('I10 delete 400 for non-parsed', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    await page.goto('/imports');
    const status = await page.evaluate(async () => {
      const resp = await fetch('/api/imports/3', { method: 'DELETE' });
      return resp.status;
    });
    expect(status).toBe(400);
  });

  test('I10b delete 404 for not found', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    await page.goto('/imports');
    const status = await page.evaluate(async () => {
      const resp = await fetch('/api/imports/999', { method: 'DELETE' });
      return resp.status;
    });
    expect(status).toBe(404);
  });

  test('I11 cancel delete', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await importsPage.getActionButton('bank.csv', 'delete').click();
    await expect(importsPage.deleteDialog).toBeVisible();
    await importsPage.deleteCancelButton.click();
    await expect(importsPage.deleteDialog).toBeHidden();
    await expect(importsPage.getRowByFilename('bank.csv')).toBeVisible();
  });

  test('I12 empty state', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: [] as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await expect(importsPage.emptyState).toBeVisible();
    await expect(importsPage.table).toBeHidden();
  });

  test('I13 skeleton pending', async ({ page }) => {
    // Delay GET /api/imports
    await page.route('**/api/imports*', async (route) => {
      if (route.request().method() === 'GET' && !route.request().url().includes('/results')) {
        await new Promise((r) => setTimeout(r, 500));
        await route.fulfill({ json: [] });
        return;
      }
      await route.fallback();
    });
    await page.route('**/api/firefly/accounts*', async (route) => {
      if (route.request().url().includes('/transactions')) await route.fallback();
      else await route.fulfill({ json: [] });
    });
    await page.route('**/health*', async (route) => route.fulfill({ json: { status: 'ok' } }));
    const importsPage = new ImportsPage(page);
    await importsPage.page.goto('/imports');
    await expect(importsPage.skeleton).toBeVisible();
  });

  test('I14 header refresh', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await expect(importsPage.rows).toHaveCount(4);
    // Mock refetch with different data
    await page.unroute('**/api/imports*');
    await page.route('**/api/imports*', async (route) => {
      const url = route.request().url();
      if (url.includes('/results')) {
        await route.fulfill({ json: fixtures.results });
        return;
      }
      if (route.request().method() === 'GET') {
        await route.fulfill({ json: baseImports.slice(0, 1) as any });
      } else await route.fallback();
    });
    await importsPage.page.getByRole('button').filter({ has: page.locator('.lucide-refresh-cw') }).first().click();
    await expect(importsPage.rows).toHaveCount(1, { timeout: 5000 });
  });

  test('I15 new import', async ({ page, mockImportsPage }) => {
    await mockImportsPage({ imports: baseImports as any });
    const importsPage = new ImportsPage(page);
    await importsPage.goto();
    await expect(importsPage.newImportButton).toBeVisible();
    await importsPage.newImportButton.click();
    // Wizard should open (check for dialog title)
    await expect(page.getByRole('heading', { name: /New import|Nueva importación/ }).first()).toBeVisible({ timeout: 3000 });
  });
});
