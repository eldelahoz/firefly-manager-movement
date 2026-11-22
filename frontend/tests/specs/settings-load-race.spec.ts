import { test, expect } from '../fixtures/api.fixture';
import { SettingsPage } from '../pages/SettingsPage';
import { fixtures } from '../fixtures/fixtures';

// T2 repro: GET /api/settings is held back until the test releases it.
const stored = {
  ...fixtures.settings,
  default_creation_tag: 'vacation',
  enable_origin_trace: false,
  default_date_format: 'dd/mm/yyyy',
};

test.describe('Settings > load race (GET /api/settings delayed)', () => {
  let release: () => void;

  test.beforeEach(async ({ page, mockSettingsPage }) => {
    await mockSettingsPage();
    const gate = new Promise<void>((resolve) => (release = resolve));
    // Registered last, so it wins; GET waits for the gate, everything else falls back to the mock.
    await page.route('**/api/settings', async (route) => {
      if (route.request().method() !== 'GET') return route.fallback();
      await gate;
      await route.fulfill({ json: stored });
    });
  });

  test('R1 edits made before settings load survive the load', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await settings.urlInput.fill('http://typed-early:8080');
    await settings.tokenInput.fill('secret-typed-early');
    await settings.tagSelect.click();
    await page.getByRole('option', { name: 'ffm-import' }).click();

    const loaded = page.waitForResponse(
      (r) => r.url().includes('/api/settings') && r.request().method() === 'GET',
    );
    release();
    await loaded;
    // Wait for a value that only comes from the loaded settings (the date format has no early edit).
    await expect(
      page.getByRole('combobox', { name: /Default date format|Formato de fecha/ }),
    ).toContainText('DD/MM/YYYY');

    await expect(settings.urlInput).toHaveValue('http://typed-early:8080');
    await expect(settings.tokenInput).toHaveValue('secret-typed-early');
    await expect(settings.tagSelect).toContainText('ffm-import');
  });

  test('R2 Save is disabled until settings load, so defaults cannot overwrite stored values', async ({
    page,
  }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await expect(settings.saveButtons).toHaveCount(3);
    for (const save of await settings.saveButtons.all()) await expect(save).toBeDisabled();
    release();
    for (const save of await settings.saveButtons.all()) await expect(save).toBeEnabled();
  });

  test('R2b Save stays disabled when GET /settings fails', async ({ page }) => {
    // Registered after the gate route, so it wins.
    await page.route('**/api/settings', async (route) => {
      if (route.request().method() !== 'GET') return route.fallback();
      await route.fulfill({ status: 500, json: { error: 'boom' } });
    });
    const settings = new SettingsPage(page);
    await settings.goto();
    await expect(page.locator('[data-sonner-toast][data-type="error"]').first()).toBeVisible();
    await expect(settings.saveButtons).toHaveCount(3);
    for (const save of await settings.saveButtons.all()) await expect(save).toBeDisabled();
  });
});
