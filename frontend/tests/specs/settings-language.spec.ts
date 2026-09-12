import { test, expect } from '../fixtures/api.fixture';
import { SettingsPage } from '../pages/SettingsPage';

test.describe('Settings > Language', () => {
  test.beforeEach(async ({ mockSettingsPage }) => {
    await mockSettingsPage();
  });

  test('S1 renders language card', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await expect(settings.languageHeading).toBeVisible();
    await expect(settings.languageSelect).toBeVisible();
  });

  test('S2 en -> es switches heading and persists', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await expect(settings.heading).toContainText('Settings');
    await settings.languageSelect.click();
    await page.getByRole('option', { name: 'Español' }).click();
    await expect.poll(async () => page.evaluate(() => localStorage.getItem('ffm.locale'))).toBe('es');
    await expect(settings.heading).toContainText('Ajustes');
    await page.reload();
    await expect(settings.heading).toContainText('Ajustes');
    await expect.poll(async () => page.evaluate(() => localStorage.getItem('ffm.locale'))).toBe('es');
    // Back to en
    await settings.languageSelect.click();
    await page.getByRole('option', { name: 'English' }).click();
    await expect(settings.heading).toContainText('Settings');
  });

  test('S3 es -> fr switches heading', async ({ page }) => {
    await page.goto('/settings');
    await page.evaluate(() => localStorage.setItem('ffm.locale', 'es'));
    await page.reload();
    const settings = new SettingsPage(page);
    await settings.heading.waitFor({ state: 'visible' });
    await expect(settings.heading).toContainText('Ajustes');
    await settings.languageSelect.click();
    await page.getByRole('option', { name: 'Français' }).click();
    await expect.poll(async () => page.evaluate(() => localStorage.getItem('ffm.locale'))).toBe('fr');
    await expect(settings.heading).toContainText('Paramètres');
  });
});
