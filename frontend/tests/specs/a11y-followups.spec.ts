import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import { fixtures } from '../fixtures/fixtures';
import { SettingsPage } from '../pages/SettingsPage';

// frontend-a11y-followups: P1 <html lang> follows the UI language, P2 Settings load error with Retry.
// (P3, the Needs you error, lives in dashboard-needs-you.spec.ts.)
const htmlLang = (page: Page) =>
  page.evaluate(() => document.documentElement.lang);

test.describe('P1 <html lang> follows the UI language', () => {
  test.beforeEach(async ({ mockSettingsPage }) => {
    await mockSettingsPage();
  });

  for (const lang of ['es', 'fr']) {
    test(`L2 persisted ${lang} is set on first load`, async ({ page }) => {
      await page.addInitScript((l) => localStorage.setItem('ffm.locale', l), lang);
      await page.goto('/settings');
      await expect.poll(() => htmlLang(page)).toBe(lang);
    });
  }

  test('L3 switching language in Settings updates it without a reload', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await expect.poll(() => htmlLang(page)).toBe('en');
    await settings.languageSelect.click();
    await page.getByRole('option', { name: 'Français' }).click();
    await expect(settings.heading).toContainText('Paramètres');
    await expect.poll(() => htmlLang(page)).toBe('fr');
    await settings.languageSelect.click();
    await page.getByRole('option', { name: 'English' }).click();
    await expect.poll(() => htmlLang(page)).toBe('en');
  });

  test.describe('browser language es-419', () => {
    test.use({ locale: 'es-419' });
    test('L4 a regional variant resolves to the bundled language (es)', async ({ page }) => {
      await page.goto('/settings');
      await expect(new SettingsPage(page).heading).toContainText('Ajustes');
      await expect.poll(() => htmlLang(page)).toBe('es');
    });
  });
});

test.describe('P2 Settings load error', () => {
  test('SE1 failed GET /api/settings shows an alert with Retry that recovers', async ({
    page,
    mockSettingsPage,
  }) => {
    await mockSettingsPage();
    const ok = fixtures.settings;
    let fail = true;
    // Registered after the fixture's route: the last matching route wins.
    await page.route('**/api/settings', async (route) => {
      if (route.request().method() !== 'GET') return route.fallback();
      if (fail) return route.fulfill({ status: 500, json: { error: 'boom' } });
      await route.fulfill({ json: ok });
    });
    const settings = new SettingsPage(page);
    await settings.goto();
    await expect(settings.heading).toBeVisible();
    // The error toast is an alert too: pick the one that holds Retry.
    const alert = page
      .getByRole('alert')
      .filter({ has: page.getByRole('button', { name: 'Retry' }) });
    await expect(alert).toBeVisible();
    await expect(alert).toContainText('Something went wrong');
    // Nothing was loaded, so nothing can be saved over the real values.
    await expect(settings.urlInput).toHaveValue('');
    for (const save of [settings.saveConnectionButton, settings.saveCreationButton])
      await expect(save).toBeDisabled();

    fail = false;
    await alert.getByRole('button', { name: 'Retry' }).click();
    await expect(alert).toHaveCount(0);
    await expect(settings.urlInput).toHaveValue(ok.ff3_url);
    for (const save of [settings.saveConnectionButton, settings.saveCreationButton])
      await expect(save).toBeEnabled();
  });

  test('SE2 Retry keeps focus and the alert while retrying; a double Enter sends one GET', async ({
    page,
    mockSettingsPage,
  }) => {
    await mockSettingsPage();
    const ok = fixtures.settings;
    // fail: immediate 500; slow-fail: 500 after 1.5 s; ok: 200 after 0.5 s (long enough for a second Enter).
    let mode: 'fail' | 'slow-fail' | 'ok' = 'fail';
    let started = 0;
    let settled = 0;
    await page.route('**/api/settings', async (route) => {
      if (route.request().method() !== 'GET') return route.fallback();
      started++;
      const m = mode;
      // Only the first request of the slow phase is slow: React Query's own one-off retry then fails fast.
      if (m === 'slow-fail') mode = 'fail';
      if (m !== 'fail') await new Promise((r) => setTimeout(r, m === 'ok' ? 500 : 1500));
      if (m === 'ok') await route.fulfill({ json: ok });
      else await route.fulfill({ status: 500, json: { error: 'boom' } });
      settled++;
    });
    const settings = new SettingsPage(page);
    await settings.goto();
    const alert = page
      .getByRole('alert')
      .filter({ has: page.getByRole('button', { name: 'Retry' }) });
    const retry = alert.getByRole('button', { name: 'Retry' });
    await expect(alert).toBeVisible();
    await expect.poll(() => settled).toBe(started); // the initial load and its own retry are done

    // A slow retry that fails again: focus and the alert must survive the whole time.
    mode = 'slow-fail';
    const before = started;
    await retry.focus();
    await page.keyboard.press('Enter');
    await expect.poll(() => started).toBeGreaterThan(before);
    await expect(alert).toBeVisible();
    await expect(retry).toBeFocused();
    await expect.poll(() => settled, { timeout: 10_000 }).toBe(started);
    await expect(alert).toBeVisible();
    await expect(retry).toBeFocused();

    // Healthy now: two quick Enters while the request is in flight make a single GET.
    mode = 'ok';
    const beforeOk = started;
    await page.keyboard.press('Enter');
    await page.keyboard.press('Enter');
    await expect(settings.urlInput).toHaveValue(ok.ff3_url);
    await expect(alert).toHaveCount(0);
    expect(started - beforeOk, 'GET /api/settings sent by the two Enters').toBe(1);
  });
});
