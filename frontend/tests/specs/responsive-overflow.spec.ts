import type { Locator, Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import { mockSettings, mockTags, mockCategories, mockBudgets } from '../mocks';
import { fixtures } from '../fixtures/fixtures';

// M0 guard (NFR-12): at phone width no page scrolls sideways and the edit dialog fits the screen.
// The shared fixtures have one-word descriptions, which never overflow; real bank lines are long,
// so this spec overrides the text-bearing endpoints with realistic lengths.

const PHONE = { width: 390, height: 844 };
const DESKTOP = { width: 1440, height: 900 };
const LONG_DESC = 'PAGO PSE ELECTRIFICADORA DEL VALLE FACTURA 2026-09 REF 88K2-SEGUNDA-ENTREGA';
const LONG_FILE = 'extracto-bancolombia-cuenta-ahorros-septiembre-2026.csv';
const withLongDesc = <T extends { description: string }>(x: T): T => ({
  ...x,
  description: LONG_DESC,
});

const r = fixtures.results;
const results = {
  ...r,
  matched: r.matched.map((m) => ({ movement: withLongDesc(m.movement), ff3: withLongDesc(m.ff3) })),
  missing: r.missing.map(withLongDesc),
  ignored: r.ignored.map(withLongDesc),
  extra: r.extra.map(withLongDesc),
};
const imports = fixtures.imports.map((b) => ({
  ...b,
  filename: LONG_FILE,
  account_name: 'Cuenta de Ahorros Bancolombia Principal',
}));

async function expectNoHorizontalOverflow(page: Page, where: string) {
  const { scrollWidth, clientWidth } = await page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    clientWidth: document.documentElement.clientWidth,
  }));
  expect(scrollWidth, `${where}: scrollWidth ${scrollWidth} vs clientWidth ${clientWidth}`).toBe(
    clientWidth,
  );
}

async function expectInsideViewport(el: Locator, what: string) {
  const vp = el.page().viewportSize()!;
  const box = await el.boundingBox();
  expect(box, `${what} bounding box`).not.toBeNull();
  const { x, y, width, height } = box!;
  expect(x, `${what} left edge`).toBeGreaterThanOrEqual(0);
  expect(y, `${what} top edge`).toBeGreaterThanOrEqual(0);
  expect(x + width, `${what} right edge`).toBeLessThanOrEqual(vp.width);
  expect(y + height, `${what} bottom edge`).toBeLessThanOrEqual(vp.height);
}

/** Horizontal fit only: vertical position depends on scroll. */
async function expectWithinViewportWidth(el: Locator, what: string) {
  await el.scrollIntoViewIfNeeded();
  const box = await el.boundingBox();
  expect(box, `${what} bounding box`).not.toBeNull();
  expect(box!.x, `${what} left edge`).toBeGreaterThanOrEqual(0);
  expect(box!.x + box!.width, `${what} right edge`).toBeLessThanOrEqual(
    el.page().viewportSize()!.width,
  );
}

/** VirtualList rows are absolutely positioned; a stale measurement makes tall (stacked) rows overlap. */
async function expectRowsDoNotOverlap(page: Page, where: string) {
  const boxes = await page
    .locator('[data-index]')
    .evaluateAll((els) =>
      els.map((e) => ({
        i: Number((e as HTMLElement).dataset.index),
        ...e.getBoundingClientRect().toJSON(),
      })),
    );
  boxes.sort((a, b) => a.top - b.top);
  for (let k = 1; k < boxes.length; k++) {
    const prev = boxes[k - 1],
      cur = boxes[k];
    expect(
      cur.top,
      `${where}: row ${cur.i} (top ${cur.top}) overlaps row ${prev.i} (bottom ${prev.bottom})`,
    ).toBeGreaterThanOrEqual(prev.bottom - 0.5);
  }
}

async function mockRealisticData(page: Page) {
  await mockSettings(page);
  await mockTags(page);
  await mockCategories(page);
  await mockBudgets(page);
  // Registered after the shared mocks, so these win (Playwright matches the newest route first).
  await page.route('**/api/imports', (route) =>
    route.request().method() === 'GET' ? route.fulfill({ json: imports }) : route.fallback(),
  );
  await page.route('**/api/imports/*/results*', (route) => route.fulfill({ json: results }));
  await page.route('**/api/imports/1', (route) =>
    route.fulfill({
      json: { batch: imports[0], movements: [], headers: [], detected_delimiter: ',' },
    }),
  );
  await page.route('**/api/firefly/accounts/*/transactions*', (route) =>
    route.fulfill({ json: fixtures.txs.map(withLongDesc) }),
  );
}

const openResults = async (page: Page) => {
  await page.goto(`/results/${fixtures.imports[0].id}`);
  await page.waitForLoadState('networkidle');
};

for (const [label, viewport] of [
  ['390x844', PHONE],
  ['1440x900', DESKTOP],
] as const) {
  test.describe(`Responsive at ${label}: no horizontal overflow (M0, NFR-12)`, () => {
    test.use({ viewport });

    test.beforeEach(async ({ page, mockDashboard }) => {
      await mockDashboard();
      await mockRealisticData(page);
    });

    test('dashboard', async ({ page }) => {
      await page.goto('/');
      await expect(page.getByText(LONG_FILE).first()).toBeVisible();
      await page.waitForLoadState('networkidle');
      await expectNoHorizontalOverflow(page, '/');
    });

    test('imports', async ({ page }) => {
      await page.goto('/imports');
      await expect(page.getByText(LONG_FILE).first()).toBeVisible();
      await expectNoHorizontalOverflow(page, '/imports');
    });

    test('results, every tab: no overflow, description visible, rows do not overlap', async ({
      page,
    }) => {
      await openResults(page);
      const tabs = page.getByRole('tab');
      await expect(tabs.first()).toBeVisible();
      const count = await tabs.count();
      expect(count).toBeGreaterThan(1);
      for (let i = 0; i < count; i++) {
        const tab = tabs.nth(i);
        const name = (await tab.textContent())?.trim() ?? `tab ${i}`;
        await tab.click();
        await expect(tab).toHaveAttribute('aria-selected', 'true');
        await expect
          .soft(page.getByText(LONG_DESC).first(), `${name}: description visible`)
          .toBeVisible();
        await expectNoHorizontalOverflow(page, `/results tab "${name}"`);
        await expectRowsDoNotOverlap(page, `/results tab "${name}"`);
      }
    });

    test('Missing: Edit, Ignore and Create are reachable inside the viewport', async ({ page }) => {
      await openResults(page);
      await page.getByRole('tab', { name: /Missing/ }).click();
      const rows = page
        .getByRole('tabpanel')
        .getByRole('row')
        .filter({ has: page.getByRole('button', { name: 'Edit' }) });
      const n = await rows.count();
      expect(n, 'missing rows with actions').toBe(results.missing.length);
      for (let i = 0; i < n; i++) {
        await expectWithinViewportWidth(
          rows.nth(i).getByRole('button', { name: 'Edit' }),
          `row ${i} Edit`,
        );
        await expectWithinViewportWidth(
          rows.nth(i).getByRole('button', { name: 'Ignore' }),
          `row ${i} Ignore`,
        );
      }
      await rows.first().getByRole('checkbox').check();
      const create = page.getByRole('button', { name: /Create in Firefly III/ });
      await expect(create).toBeEnabled();
      await expectWithinViewportWidth(create, 'Create button');
    });

    test('edit dialog fits the viewport', async ({ page }) => {
      await openResults(page);
      await page.getByRole('tab', { name: /Missing/ }).click();
      await page.getByRole('button', { name: 'Edit' }).first().click();
      const dialog = page.getByRole('dialog');
      await expect(dialog).toBeVisible();
      // The sheet slides in: wait until its box stops moving before measuring.
      let prev = '';
      await expect
        .poll(async () => {
          const cur = JSON.stringify(await dialog.boundingBox());
          const same = cur === prev;
          prev = cur;
          return same;
        })
        .toBe(true);
      await expectInsideViewport(dialog, 'dialog');
      // The box can fit while its content spills out (cut-off close button / amount).
      const { scrollWidth, clientWidth } = await dialog.evaluate((d) => ({
        scrollWidth: d.scrollWidth,
        clientWidth: d.clientWidth,
      }));
      expect(
        scrollWidth,
        `dialog content scrollWidth ${scrollWidth} vs clientWidth ${clientWidth}`,
      ).toBeLessThanOrEqual(clientWidth);
      await expectInsideViewport(
        dialog.getByRole('button', { name: /close/i }).first(),
        'dialog close button',
      );
    });
  });
}
