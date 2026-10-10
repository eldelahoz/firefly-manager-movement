import { test, expect } from '../fixtures/api.fixture';
import { mockSettings } from '../mocks';
import { SettingsPage } from '../pages/SettingsPage';

test.describe('Settings > Connection', () => {
  test.beforeEach(async ({ mockSettingsPage }) => {
    await mockSettingsPage();
  });

  test('S1 renders connection with mocked url and has_token hint', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await expect(settings.urlInput).toHaveValue('http://localhost:8080');
    await expect(settings.tokenInput).toHaveValue('');
    await expect(settings.tokenKeepHint).toBeVisible();
    await expect(settings.testConnectionButton).toBeVisible();
  });

  test('S2 verify success shows Connection OK', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await settings.fillConnection('http://localhost:8080', 'tok123');
    await settings.testConnectionButton.click();
    await settings.expectToast('Connection OK (Firefly III 6.1.36)');
  });

  test('S3 verify 400 invalid url shows error', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await page.unroute('**/api/settings/verify');
    await page.route('**/api/settings/verify', async (route) => {
      await route.fulfill({ status: 400, json: { error: 'invalid url' } });
    });
    await settings.fillConnection('not-a-url', 'tok');
    await settings.testConnectionButton.click();
    await expect(page.getByText(/invalid url/i).first()).toBeVisible({ timeout: 5000 });
  });

  test('S4 verify 502 unreachable shows error', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await page.unroute('**/api/settings/verify');
    await page.route('**/api/settings/verify', async (route) => {
      await route.fulfill({ status: 502, json: { error: 'unreachable' } });
    });
    await settings.fillConnection('http://localhost:8080', 'tok');
    await settings.testConnectionButton.click();
    await expect(page.getByText(/unreachable/i).first()).toBeVisible({ timeout: 5000 });
  });

  test('S5 save persists and shows Settings saved', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await settings.fillConnection('http://ff3.new', 'tok2');
    const putPromise = page.waitForResponse(
      (r) => r.url().includes('/api/settings') && r.request().method() === 'PUT',
    );
    await settings.saveConnectionButton.click();
    const putReq = await putPromise;
    expect(putReq.request().postDataJSON()).toMatchObject({ ff3_url: 'http://ff3.new' });
    await settings.expectToast('Settings saved');
  });

  test('S6 save keeps token when empty', async ({ page }) => {
    const settings = new SettingsPage(page);
    await settings.goto();
    await settings.urlInput.fill('http://ff3.new');
    // token left empty -> should keep has_token true
    const putPromise = page.waitForResponse(
      (r) => r.url().includes('/api/settings') && r.request().method() === 'PUT',
    );
    await settings.saveConnectionButton.click();
    const req = await putPromise;
    const body = req.request().postDataJSON() as Record<string, unknown>;
    expect(body.ff3_token).toBeUndefined();
    await settings.expectToast('Settings saved');
  });

  test('S7 refetches FF3 tags after saving a new connection', async ({ page }) => {
    const settings = new SettingsPage(page);
    // Gate the tags list on "a PUT has been received": the refetch (triggered by
    // the save's onSuccess) always arrives after the PUT was handled, so the
    // flag is set deterministically, unlike waiting on the test-side promise.
    let saved = false;
    await page.unroute('**/api/settings');
    await mockSettings(page, {
      onPut: () => {
        saved = true;
      },
    });
    await page.unroute('**/api/firefly/tags');
    await page.route('**/api/firefly/tags', async (route) => {
      await route.fulfill({ json: saved ? ['new-connection-tag'] : ['ffm-import', 'vacation'] });
    });
    await settings.goto();

    await settings.fillConnection('http://ff3.new', 'tok2');
    const putPromise = page.waitForResponse(
      (r) => r.url().includes('/api/settings') && r.request().method() === 'PUT',
    );
    await settings.saveConnectionButton.click();
    await putPromise;

    // Tags must refresh from the (now different) connection without a page reload.
    await settings.tagSelect.click();
    await expect(page.getByRole('option', { name: 'new-connection-tag' })).toBeVisible();
    await expect(page.getByRole('option', { name: 'vacation' })).toBeHidden();
  });
});
