import type { Locator, Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import { DashboardPage } from '../pages/DashboardPage';
import { SettingsPage } from '../pages/SettingsPage';
import { mockSettings, mockTags, mockCategories, mockBudgets } from '../mocks';
import { fixtures } from '../fixtures/fixtures';

// NFR-08 v0.1: number format preference (criteria 1-8). Amounts are asserted on textContent so the
// exact characters count (U+202F in space-comma, true minus U+2212) — toHaveText would fold them.

const FORMAT_KEY = 'ffm.numberFormat';
const NNBSP = ' ';

type Format = 'comma-dot' | 'dot-comma' | 'space-comma' | 'apostrophe-dot' | 'comma-dot-in';

// 1234567.89 and 2500.00 per format (NFR-08 table).
const FORMATS: { value: Format; big: string; small: string; decimal: string }[] = [
  { value: 'comma-dot', big: '1,234,567.89', small: '2,500.00', decimal: '.' },
  { value: 'dot-comma', big: '1.234.567,89', small: '2.500,00', decimal: ',' },
  {
    value: 'space-comma',
    big: `1${NNBSP}234${NNBSP}567,89`,
    small: `2${NNBSP}500,00`,
    decimal: ',',
  },
  { value: 'apostrophe-dot', big: "1'234'567.89", small: "2'500.00", decimal: '.' },
  { value: 'comma-dot-in', big: '12,34,567.89', small: '2,500.00', decimal: '.' },
];

// The unit is the CLDR currency unit as before this feature: COP stays COP, EUR is the euro sign.
const UNIT: Record<string, string> = { COP: 'COP', EUR: '€' };

const BIG_CENTS = 123456789;
const detailFor = (currency: string) => ({
  batch: { ...fixtures.imports[0], currency },
  movements: [],
  headers: [],
  detected_delimiter: ',',
});
const movement = (id: number, description: string, amount_cents: number) => ({
  id,
  line_no: id,
  date: '2026-08-02',
  amount_cents,
  description,
  status: 'missing',
});
const baseResults = {
  reconciliation_id: 1,
  matched: [],
  missing: [movement(12, 'Book', -BIG_CENTS), movement(13, 'Salary', 250000)],
  ignored: [],
  extra: [],
};

async function mockResults(page: Page, currency: string, results: unknown = baseResults) {
  await page.route('**/api/imports/1', (r) => r.fulfill({ json: detailFor(currency) }));
  await page.route('**/api/imports/1/results', (r) => r.fulfill({ json: results }));
}

const rowFor = (page: Page, text: string) => page.getByRole('row').filter({ hasText: text });
/** The money cell of a results row: the one that ends in the batch currency. */
const amountCell = (row: Locator, currency: string) =>
  row.getByRole('cell').filter({ hasText: new RegExp(`\\d\\s${UNIT[currency]}$`) });
const text = (loc: Locator) => async () => (await loc.textContent()) ?? '';
const setStored = (page: Page, key: string, value: string) =>
  page.addInitScript(([k, v]) => localStorage.setItem(k, v), [key, value]);

test.describe('NFR-08 > number format', () => {
  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
  });

  // Criteria 3 + 5: one test per explicit format, rows (negative and positive) and summary, COP and EUR.
  for (const currency of ['COP', 'EUR']) {
    for (const f of FORMATS) {
      test(`NF1 ${f.value}: row and summary in ${currency}`, async ({ page }) => {
        await setStored(page, FORMAT_KEY, f.value);
        await mockResults(page, currency);
        await page.goto('/results/1');
        await expect(page.getByRole('tab', { name: /Missing/ })).toBeVisible();

        const book = amountCell(rowFor(page, 'Book'), currency);
        const salary = amountCell(rowFor(page, 'Salary'), currency);
        await expect.poll(text(book)).toBe(`−${f.big} ${UNIT[currency]}`);
        await expect.poll(text(salary)).toBe(`+${f.small} ${UNIT[currency]}`);

        const summary = page.getByRole('region', { name: 'Reconciliation summary' });
        await expect
          .poll(text(summary))
          .toContain(`${f.big} ${UNIT[currency]} out · ${f.small} ${UNIT[currency]} in`);
      });
    }
  }

  test('NF2 an explicit format wins over the UI language (es + comma-dot)', async ({ page }) => {
    await setStored(page, 'ffm.locale', 'es');
    await setStored(page, FORMAT_KEY, 'comma-dot');
    await mockResults(page, 'COP');
    await page.goto('/results/1');
    await expect.poll(text(amountCell(rowFor(page, 'Book'), 'COP'))).toBe('−1,234,567.89 COP');
    await expect
      .poll(text(page.getByRole('region', { name: 'Resumen de la conciliación' })))
      .toContain('1,234,567.89 COP');
  });

  // Criterion 2
  for (const [lang, stored, expected] of [
    ['en', undefined, '−1,234,567.89 COP'],
    ['es', undefined, '−1.234.567,89 COP'],
    ['es', 'auto', '−1.234.567,89 COP'],
    ['en', 'auto', '−1,234,567.89 COP'],
  ] as const) {
    test(`NF3 auto follows the UI language (${lang}, stored=${stored ?? 'none'})`, async ({
      page,
    }) => {
      await setStored(page, 'ffm.locale', lang);
      if (stored) await setStored(page, FORMAT_KEY, stored);
      await mockResults(page, 'COP');
      await page.goto('/results/1');
      await expect.poll(text(amountCell(rowFor(page, 'Book'), 'COP'))).toBe(expected);
    });
  }

  // Criterion 6 + the 4-digit case of criterion 5 (123456 cents = 1234.56)
  test('NF4 a 4-digit integer part is grouped in es auto (1.234,50, not 1234,50)', async ({
    page,
  }) => {
    await setStored(page, 'ffm.locale', 'es');
    await mockResults(page, 'COP', { ...baseResults, missing: [movement(12, 'Book', -123450)] });
    await page.goto('/results/1');
    await expect.poll(text(amountCell(rowFor(page, 'Book'), 'COP'))).toBe('−1.234,50 COP');
  });

  for (const [format, expected] of [
    ['comma-dot', '1,234.56'],
    ['dot-comma', '1.234,56'],
    ['space-comma', `1${NNBSP}234,56`],
    ['apostrophe-dot', "1'234.56"],
    ['comma-dot-in', '1,234.56'],
  ] as const) {
    test(`NF5 ${format}: 123456 cents renders ${expected.replace(NNBSP, '<U+202F>')}`, async ({
      page,
    }) => {
      await setStored(page, 'ffm.locale', 'es'); // es auto would differ: the explicit format must win
      await setStored(page, FORMAT_KEY, format);
      await mockResults(page, 'EUR', { ...baseResults, missing: [movement(12, 'Book', 123456)] });
      await page.goto('/results/1');
      await expect.poll(text(amountCell(rowFor(page, 'Book'), 'EUR'))).toBe(`+${expected} €`);
    });
  }

  // One test per format: each gets a fresh context, so storage never depends on init-script order.
  for (const [format, expected] of [
    ['comma-dot', '1,000.00'],
    ['dot-comma', '1.000,00'],
    ['space-comma', `1${NNBSP}000,00`],
    ['apostrophe-dot', "1'000.00"],
    ['comma-dot-in', '1,000.00'],
  ] as const) {
    test(`NF6 ${format}: whole units keep two decimals and 4 digits group (${expected.replace(NNBSP, '<U+202F>')})`, async ({
      page,
    }) => {
      await mockResults(page, 'EUR', { ...baseResults, missing: [movement(12, 'Book', 100000)] });
      await page.goto('/results/1');
      await page.evaluate(([k, v]) => localStorage.setItem(k, v), [FORMAT_KEY, format]);
      await page.reload();
      await expect.poll(text(amountCell(rowFor(page, 'Book'), 'EUR'))).toBe(`+${expected} €`);
    });
  }

  // Criterion 4
  test('NF7 the choice persists after reload and an invalid stored value falls back to auto', async ({
    page,
  }) => {
    await mockResults(page, 'COP');
    await page.goto('/settings');
    const settings = new SettingsPage(page);
    await settings.numberFormatSelect.click();
    await page.getByRole('option', { name: '1.234.567,89', exact: true }).click();
    await expect
      .poll(() => page.evaluate((k) => localStorage.getItem(k), FORMAT_KEY))
      .toBe('dot-comma');
    await page.reload();
    await expect(settings.numberFormatSelect).toContainText('1.234.567,89');
    await page.goto('/results/1');
    await expect.poll(text(amountCell(rowFor(page, 'Book'), 'COP'))).toBe('−1.234.567,89 COP');
    await page.reload();
    await expect.poll(text(amountCell(rowFor(page, 'Book'), 'COP'))).toBe('−1.234.567,89 COP');

    // Garbage in storage: behaves as auto (en -> 1,234,567.89), the app does not break.
    await page.evaluate((k) => localStorage.setItem(k, 'not-a-format'), FORMAT_KEY);
    await page.reload();
    await expect.poll(text(amountCell(rowFor(page, 'Book'), 'COP'))).toBe('−1,234,567.89 COP');
  });

  test('NF8 an invalid stored value follows the UI language like auto (es)', async ({ page }) => {
    await setStored(page, 'ffm.locale', 'es');
    await setStored(page, FORMAT_KEY, '{"oops":1}');
    await mockResults(page, 'COP');
    await page.goto('/results/1');
    await expect.poll(text(amountCell(rowFor(page, 'Book'), 'COP'))).toBe('−1.234.567,89 COP');
  });

  // Criterion 7
  test('NF9 the decimal separator and cents are muted; text content and accessible name are unchanged', async ({
    page,
  }) => {
    await setStored(page, FORMAT_KEY, 'dot-comma');
    await mockResults(page, 'COP');
    await page.goto('/results/1');
    const salary = amountCell(rowFor(page, 'Salary'), 'COP');
    await expect.poll(text(salary)).toBe('+2.500,00 COP');

    // muted part: separator + cents only, with a colour different from the integer part
    const fraction = salary.locator('[data-part="fraction"]');
    await expect(fraction).toHaveCount(1);
    await expect.poll(text(fraction)).toBe(',00');
    const colors = await salary.evaluate((cell) => {
      const frac = cell.querySelector('[data-part="fraction"]') as HTMLElement;
      const root = frac.parentElement as HTMLElement;
      return [getComputedStyle(frac).color, getComputedStyle(root).color];
    });
    expect(colors[0], 'fraction colour differs from the amount colour').not.toBe(colors[1]);

    // nothing hidden from assistive tech, and the cell reads as the plain amount
    await expect(salary.locator('[aria-hidden="true"]')).toHaveCount(0);
    await expect(salary).toMatchAriaSnapshot('- cell "+2.500,00 COP"');
  });

  test('NF9b the muted fraction follows the format (comma-dot: .89)', async ({ page }) => {
    await setStored(page, FORMAT_KEY, 'comma-dot');
    await mockResults(page, 'COP');
    await page.goto('/results/1');
    const book = amountCell(rowFor(page, 'Book'), 'COP');
    await expect.poll(text(book.locator('[data-part="fraction"]'))).toBe('.89');
    await expect.poll(text(book)).toBe('−1,234,567.89 COP');
  });

  // Criterion 1
  test('NF10 Settings offers the 6 values labelled with their example', async ({
    page,
    mockSettingsPage,
  }) => {
    await mockSettingsPage();
    const settings = new SettingsPage(page);
    await settings.goto();
    await settings.numberFormatSelect.click();
    const options = page.getByRole('option');
    await expect(options).toHaveCount(6);
    // each explicit option is labelled with its example only; Auto shows what it equals (en)
    const labels = (await options.allTextContents()).map((l) => l.trim());
    for (const f of FORMATS) expect(labels, `option labelled ${f.big}`).toContain(f.big);
    expect(labels).toContain('Auto (1,234,567.89)');
  });

  test('NF11 the Settings preview follows the selected format, with no Save', async ({
    page,
    mockSettingsPage,
  }) => {
    await mockSettingsPage();
    const settings = new SettingsPage(page);
    await settings.goto();
    const preview = settings.numberFormatPreview;
    await expect(preview).toBeVisible();
    const seen = new Set<string>();
    for (const f of FORMATS) {
      await settings.numberFormatSelect.click();
      // exact: the Auto option also carries an example ("Auto (1,234,567.89)")
      await page.getByRole('option', { name: f.big, exact: true }).click();
      // the character before the two cents is the decimal separator of the chosen format
      await expect
        .poll(text(preview))
        .toMatch(new RegExp(`\\d${f.decimal === '.' ? '\\.' : ','}\\d{2}(?!\\d)`));
      const shown = await text(preview)();
      expect(shown, 'preview has no ungrouped 4+ digit run').not.toMatch(/\d{4,}[.,]\d{2}/);
      if (f.value === 'space-comma') expect(shown).toContain(NNBSP);
      if (f.value === 'apostrophe-dot') expect(shown).toMatch(/\d['’]\d{3}/);
      seen.add(shown);
      await expect
        .poll(() => page.evaluate((k) => localStorage.getItem(k), FORMAT_KEY))
        .toBe(f.value);
    }
    expect(seen.size, 'each format previews differently').toBeGreaterThanOrEqual(4);
  });
});

test.describe('NFR-08 > auto with a regional browser language', () => {
  test.use({ locale: 'es-419' });

  test('NF14 no ffm.locale and es-419 browser: Auto formats like es (1.234,56)', async ({
    page,
    mockImportsPage,
  }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
    await mockResults(page, 'COP', { ...baseResults, missing: [movement(12, 'Book', -123456)] });
    await page.goto('/results/1');
    // i18next caches the detected code afterwards, so nothing was stored before this first visit
    expect(await page.evaluate(() => navigator.language)).toBe('es-419');
    await expect.poll(text(amountCell(rowFor(page, 'Book'), 'COP'))).toBe('−1.234,56 COP');
  });
});

test.describe('NFR-08 > number format on the dashboard', () => {
  test.beforeEach(async ({ page, mockDashboard }) => {
    await page.clock.setFixedTime(new Date('2026-09-09T12:00:00Z'));
    await mockSettings(page);
    await mockTags(page);
    await mockDashboard();
  });

  test('NF12 the Firefly III browse list follows a stored format', async ({ page }) => {
    await setStored(page, FORMAT_KEY, 'dot-comma');
    const dashboard = new DashboardPage(page);
    await dashboard.goto();
    await dashboard.selectAccount('Checking');
    const salary = dashboard.browseList.getByRole('listitem').filter({ hasText: 'Salary' });
    await expect.poll(text(salary)).toContain('+2.500,00 €');
  });

  // Criteria 3 + 8: change in Settings, move around the SPA with no reload, no backend write, dates intact.
  test('NF13 changing the format applies at once, keeps language and dates, calls no backend', async ({
    page,
  }) => {
    const writes: string[] = [];
    page.on('request', (r) => {
      if (r.url().includes('/api/') && r.method() !== 'GET')
        writes.push(`${r.method()} ${r.url()}`);
    });
    const dashboard = new DashboardPage(page);
    const salary = () => dashboard.browseList.getByRole('listitem').filter({ hasText: 'Salary' });
    const nav = (name: string) => page.getByRole('link', { name, exact: true }).first();

    await dashboard.goto();
    await page.evaluate(() => {
      (window as unknown as { __loaded: boolean }).__loaded = true;
    });
    await dashboard.selectAccount('Checking');
    await expect.poll(text(salary())).toContain('+2,500.00 €');
    const dateBefore = (await text(salary())()).replace('+2,500.00 €', '');

    await nav('Settings').click();
    const settings = new SettingsPage(page);
    await expect(settings.heading).toContainText('Settings');
    const requestsBefore = writes.length;
    await settings.numberFormatSelect.click();
    // every /api/ request from the click until the change is fully applied, GETs included
    const afterClick: string[] = [];
    page.on('request', (r) => {
      if (r.url().includes('/api/')) afterClick.push(`${r.method()} ${r.url()}`);
    });
    await page.getByRole('option', { name: '1.234.567,89', exact: true }).click();
    await expect
      .poll(() => page.evaluate((k) => localStorage.getItem(k), FORMAT_KEY))
      .toBe('dot-comma');
    await expect.poll(text(settings.numberFormatPreview)).toMatch(/\d,\d{2}(?!\d)/);
    await expect(settings.heading).toContainText('Settings'); // language unchanged
    expect(afterClick, 'no /api/ request (GET or write) when the format changes').toEqual([]);
    expect(await page.evaluate(() => localStorage.getItem('ffm.locale'))).not.toBe('es');

    afterClick.length = 0;
    await nav('Dashboard').click();
    await dashboard.selectAccount('Checking');
    await expect.poll(text(salary())).toContain('+2.500,00 €');
    expect((await text(salary())()).replace('+2.500,00 €', '')).toBe(dateBefore);
    expect(
      await page.evaluate(() => (window as unknown as { __loaded?: boolean }).__loaded),
      'no full page reload happened',
    ).toBe(true);
    expect(writes.slice(requestsBefore), 'no write request on a format change').toEqual([]);
  });
});
