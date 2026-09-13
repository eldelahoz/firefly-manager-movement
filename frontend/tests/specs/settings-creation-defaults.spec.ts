import { test, expect } from '../fixtures/api.fixture';
import { SettingsPage } from '../pages/SettingsPage';

test.describe('Settings > Creation defaults', () => {
  test.beforeEach(async ({ mockSettingsPage }) => {
    await mockSettingsPage();
  });

  test('S1 tag dropdown shows tags', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await settings.tagSelect.click();
    await expect(page.getByRole('option', { name: 'ffm-import' })).toBeVisible();
    await expect(page.getByRole('option', { name: 'vacation' })).toBeVisible();
    await expect(page.getByRole('option', { name: /No default tag|Sin etiqueta/ })).toBeVisible();
  });

  test('S2 select tag and save sends default_creation_tag', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await settings.tagSelect.click();
    await page.getByRole('option', { name: 'vacation' }).click();
    const putPromise = page.waitForResponse(
      (r) => r.url().includes('/api/settings') && r.request().method() === 'PUT',
    );
    await settings.saveCreationButton.click();
    const body = (await putPromise).request().postDataJSON() as Record<string, unknown>;
    expect(body.default_creation_tag).toBe('vacation');
    await settings.expectToast('Settings saved');
  });

  test('S3 select none sends null', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    // First select a tag to ensure change
    await settings.tagSelect.click();
    await page.getByRole('option', { name: 'vacation' }).click();
    // Now select none
    await settings.tagSelect.click();
    await page.getByRole('option', { name: /No default tag|Sin etiqueta/ }).click();
    const putPromise = page.waitForResponse(
      (r) => r.url().includes('/api/settings') && r.request().method() === 'PUT',
    );
    await settings.saveCreationButton.click();
    const body = (await putPromise).request().postDataJSON() as Record<string, unknown>;
    expect(body.default_creation_tag).toBeNull();
    await settings.expectToast('Settings saved');
  });

  test('S4 origin trace toggle sends enable_origin_trace', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    const initial = await settings.originTraceSwitch.isChecked();
    await settings.originTraceSwitch.click();
    const putPromise = page.waitForResponse(
      (r) => r.url().includes('/api/settings') && r.request().method() === 'PUT',
    );
    await settings.saveCreationButton.click();
    const body = (await putPromise).request().postDataJSON() as Record<string, unknown>;
    expect(body.enable_origin_trace).toBe(!initial);
    await settings.expectToast('Settings saved');
  });
});
