import { test, expect } from '../fixtures/api.fixture';
import { SettingsPage } from '../pages/SettingsPage';

test.describe('Settings > Appearance', () => {
  test.beforeEach(async ({ mockSettingsPage }) => {
    await mockSettingsPage();
  });

  test('S1 renders appearance card', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await expect(settings.heading).toBeVisible();
    await expect(settings.appearanceHeading).toBeVisible();
    await expect(settings.themeLightButton).toBeVisible();
    await expect(settings.themeDarkButton).toBeVisible();
  });

  test('S2 light -> dark applies class and persists', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    // Ensure start from light
    await settings.themeLightButton.click();
    await expect.poll(async () => page.evaluate(() => localStorage.getItem('ffm.theme'))).toBe('light');
    await expect.poll(async () => page.evaluate(() => document.documentElement.classList.contains('dark'))).toBe(false);

    await settings.themeDarkButton.click();
    await expect.poll(async () => page.evaluate(() => document.documentElement.classList.contains('dark'))).toBe(true);
    await expect.poll(async () => page.evaluate(() => localStorage.getItem('ffm.theme'))).toBe('dark');

    await page.reload();
    await settings.appearanceHeading.waitFor({ state: 'visible' });
    await expect.poll(async () => page.evaluate(() => document.documentElement.classList.contains('dark'))).toBe(true);
    await expect.poll(async () => page.evaluate(() => localStorage.getItem('ffm.theme'))).toBe('dark');

    // Back to light
    await settings.themeLightButton.click();
    await expect.poll(async () => page.evaluate(() => document.documentElement.classList.contains('dark'))).toBe(false);
    await expect.poll(async () => page.evaluate(() => localStorage.getItem('ffm.theme'))).toBe('light');
  });

  test('S3 dark persists after reload', async ({ page }) => {
    await page.goto('/settings');
    await page.evaluate(() => localStorage.setItem('ffm.theme', 'dark'));
    await page.reload();
    const settings = new SettingsPage(page);
    await settings.appearanceHeading.waitFor({ state: 'visible' });
    await expect.poll(async () => page.evaluate(() => document.documentElement.classList.contains('dark'))).toBe(true);
    await expect.poll(async () => page.evaluate(() => localStorage.getItem('ffm.theme'))).toBe('dark');
  });
});
