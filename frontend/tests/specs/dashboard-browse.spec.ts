import { test, expect } from '../fixtures/api.fixture';
import { DashboardPage } from '../pages/DashboardPage';
import { fixtures } from '../fixtures/fixtures';

const { txs } = fixtures;

test.describe('Dashboard > BrowseWidget', () => {
  test.beforeEach(async ({ mockDashboard }) => {
    await mockDashboard();
  });

  test('R1.1 Selection shows transactions', async ({ page }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();

    // Default state: no account selected — paragraph shows hint
    await expect(dashboard.browseSelectAccount).toBeVisible();

    // Set up response listener BEFORE selecting account
    const responsePromise = page.waitForResponse(
      (r) =>
        r.url().includes('/api/firefly/accounts/1/transactions') &&
        r.request().method() === 'POST',
    );

    // Select "Checking" account (id=1)
    await dashboard.selectAccount('Checking');

    // Verify POST was intercepted with correct id
    const response = await responsePromise;
    expect(response.status()).toBe(200);

    // Verify list renders with 3 transactions
    const rows = await dashboard.getTransactionRows();
    expect(rows).toHaveLength(3);

    // Verify each row contains expected data
    const texts = await Promise.all(rows.map(dashboard.getTransactionText));
    expect(texts[0]).toContain('Coffee');
    expect(texts[0]).toContain('42');
    expect(texts[1]).toContain('Duplicate A');
    expect(texts[2]).toContain('Salary');
  });

  test('R1.2 "From" filter sends correct start date', async ({ page }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();

    // Select account first — set up response listener before click
    const firstResponse = page.waitForResponse(
      (r) => r.url().includes('/api/firefly/accounts/1/transactions'),
    );
    await dashboard.selectAccount('Checking');
    await firstResponse;

    // Set up listener for the next POST triggered by date change
    const dateResponse = page.waitForResponse(
      (r) =>
        r.url().includes('/api/firefly/accounts/1/transactions') &&
        r.request().method() === 'POST',
    );

    // Change "From" date to 2026-09-04 (default end is 2026-09-09)
    // Only Salary (2026-09-05) fits in [09-04, 09-09]
    await dashboard.setFromDate('2026-09-04');

    const response = await dateResponse;
    const body = response.request().postDataJSON();
    expect(body.start).toBe('2026-09-04');

    // Mock returns filtered results (date >= 2026-09-04), only Salary
    const rows = await dashboard.getTransactionRows();
    expect(rows).toHaveLength(1);
    const text = await dashboard.getTransactionText(rows[0]);
    expect(text).toContain('Salary');
  });

  test('R1.3 "To" filter sends correct end date', async ({ page }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();

    const firstResponse = page.waitForResponse(
      (r) => r.url().includes('/api/firefly/accounts/1/transactions'),
    );
    await dashboard.selectAccount('Checking');
    await firstResponse;

    const dateResponse = page.waitForResponse(
      (r) =>
        r.url().includes('/api/firefly/accounts/1/transactions') &&
        r.request().method() === 'POST',
    );

    // Change "To" date to 2026-09-02 (default start is 2026-09-01)
    // Only Coffee (2026-09-01) fits in [09-01, 09-02]
    await dashboard.setToDate('2026-09-02');

    const response = await dateResponse;
    const body = response.request().postDataJSON();
    expect(body.end).toBe('2026-09-02');

    // Only Coffee (2026-09-01) fits
    const rows = await dashboard.getTransactionRows();
    expect(rows).toHaveLength(1);
    const text = await dashboard.getTransactionText(rows[0]);
    expect(text).toContain('Coffee');
  });

  test('R1.4 Clearing filters shows all transactions', async ({ page }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();

    const firstResponse = page.waitForResponse(
      (r) => r.url().includes('/api/firefly/accounts/1/transactions'),
    );
    await dashboard.selectAccount('Checking');
    await firstResponse;

    // 1. Default range [Sept 1, Sept 9] → 3 transactions
    const rows = await dashboard.getTransactionRows();
    expect(rows).toHaveLength(3);

    // 2. Narrow From to Sept 4 → only Salary (Sept 5) fits
    const narrowResponse = page.waitForResponse(
      (r) =>
        r.url().includes('/api/firefly/accounts/1/transactions') &&
        r.request().method() === 'POST',
    );
    await dashboard.setFromDate('2026-09-04');
    await narrowResponse;
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(1, { timeout: 5000 });

    // 3. Widen From back to Sept 1 → all 3 transactions show again
    // React Query may use cache, so just wait for the list count to update
    await dashboard.setFromDate('2026-09-01');
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(3, { timeout: 5000 });
  });

  test('R1.4b Clearing date input with Delete does not crash (regression: invalid date)', async ({
    page,
  }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();
    const firstResp = page.waitForResponse((r) => r.url().includes('/api/firefly/accounts/1/transactions'));
    await dashboard.selectAccount('Checking');
    await firstResp;

    // Focus From input, select all and press Delete → value becomes ""
    // This is the manual reproduction: pararse en el input y apretar Del
    await dashboard.fromInput.click();
    await dashboard.fromInput.press('ControlOrMeta+A');
    await dashboard.fromInput.press('Delete');

    // Should not throw Unexpected Application Error! (React error boundary)
    await expect(page.getByText('Unexpected Application Error')).toBeHidden({ timeout: 2000 });
    // Input should be empty (cleared)
    await expect(dashboard.fromInput).toHaveValue('');
    // No POST with empty start should have been sent — page should not crash, list stays visible
    // (query is disabled when start is invalid, so no new request)
    await expect(dashboard.browseList).toBeVisible();

    // Same for To input
    await dashboard.toInput.click();
    await dashboard.toInput.press('ControlOrMeta+A');
    await dashboard.toInput.press('Delete');
    await expect(page.getByText('Unexpected Application Error')).toBeHidden({ timeout: 2000 });
    await expect(dashboard.toInput).toHaveValue('');
    await expect(dashboard.browseList).toBeVisible();

    // Recovery: setting a valid date again should re-enable query and show results
    await dashboard.setFromDate('2026-09-01');
    await dashboard.setToDate('2026-09-09');
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(3, { timeout: 5000 });
  });

  test('R1.4c Filtering to 2026-09-03 shows 2 then clearing restores all', async ({
    page,
  }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();
    const firstResp = page.waitForResponse((r) => r.url().includes('/api/firefly/accounts/1/transactions'));
    await dashboard.selectAccount('Checking');
    await firstResp;

    // Initial 3
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(3, { timeout: 5000 });

    // Set From to 2026-09-03 → 2 (Duplicate A 03 + Salary 05), Coffee 01 filtered out
    const to03Resp = page.waitForResponse(
      (r) => r.url().includes('/api/firefly/accounts/1/transactions') && r.request().method() === 'POST',
    );
    await dashboard.setFromDate('2026-09-03');
    const resp03 = await to03Resp;
    expect(resp03.request().postDataJSON().start).toBe('2026-09-03');
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(2, { timeout: 5000 });
    const texts03 = await Promise.all(
      (await dashboard.getTransactionRows()).map((r) => dashboard.getTransactionText(r)),
    );
    expect(texts03.some((t) => t.includes('Duplicate A'))).toBeTruthy();
    expect(texts03.some((t) => t.includes('Salary'))).toBeTruthy();
    expect(texts03.some((t) => t.includes('Coffee'))).toBeFalsy();

    // Clear via Delete → input empty, query disabled (keepPreviousData), list keeps 2, no crash
    await dashboard.fromInput.click();
    await dashboard.fromInput.press('ControlOrMeta+A');
    await dashboard.fromInput.press('Delete');
    await expect(page.getByText('Unexpected Application Error')).toBeHidden({ timeout: 2000 });
    await expect(dashboard.fromInput).toHaveValue('');
    await expect(dashboard.browseList).toBeVisible();
    // keepPreviousData keeps previous 2 visible (no new fetch)
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(2, { timeout: 5000 });

    // Recovery: set back to 2026-09-01 → 3 (cached, may not refetch, just wait for count)
    await dashboard.setFromDate('2026-09-01');
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(3, { timeout: 5000 });
  });

  test('R1.7 Open range: From empty + To shows all until To', async ({ page }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();
    const firstResp = page.waitForResponse((r) => r.url().includes('/api/firefly/accounts/1/transactions'));
    await dashboard.selectAccount('Checking');
    await firstResp;
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(3, { timeout: 5000 });

    // Clear From (open start) and set To to 2026-09-03 → 2 (Coffee 01 + Duplicate 03)
    await dashboard.fromInput.click();
    await dashboard.fromInput.press('ControlOrMeta+A');
    await dashboard.fromInput.press('Delete');
    await expect(dashboard.fromInput).toHaveValue('');
    const to03Resp = page.waitForResponse(
      (r) => r.url().includes('/api/firefly/accounts/1/transactions') && r.request().method() === 'POST',
    );
    await dashboard.setToDate('2026-09-03');
    const resp = await to03Resp;
    const body = resp.request().postDataJSON();
    expect(body.start).toBeUndefined();
    expect(body.end).toBe('2026-09-03');
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(2, { timeout: 5000 });
    const texts = await Promise.all((await dashboard.getTransactionRows()).map((r) => dashboard.getTransactionText(r)));
    expect(texts.some((t) => t.includes('Coffee'))).toBeTruthy();
    expect(texts.some((t) => t.includes('Duplicate A'))).toBeTruthy();
    expect(texts.some((t) => t.includes('Salary'))).toBeFalsy();

    // Open end: From 2026-09-03 + To empty → 2 (Duplicate 03 + Salary 05)
    await dashboard.fromInput.fill('2026-09-03');
    await dashboard.toInput.click();
    await dashboard.toInput.press('ControlOrMeta+A');
    await dashboard.toInput.press('Delete');
    await expect(dashboard.toInput).toHaveValue('');
    // With From=03 and To empty, should show 2 (03,05) - open end
    await expect(dashboard.browseList.getByRole('listitem')).toHaveCount(2, { timeout: 5000 });
  });

  test('R1.5 States: empty, pending, error, empty results', async ({ page }) => {
    const dashboard = new DashboardPage(page);

    // 1. No account selected → shows "Select an account" paragraph
    await page.route('**/api/firefly/accounts', (route) =>
      route.fulfill({ json: fixtures.accounts }),
    );
    await page.route('**/api/firefly/accounts/*/transactions', (route) =>
      route.fulfill({ json: txs }),
    );
    await page.route('**/health', (route) => route.fulfill({ json: { status: 'ok' } }));
    await page.route('**/api/imports', (route) => route.fulfill({ json: fixtures.imports }));
    await dashboard.goto();
    await expect(dashboard.browseSelectAccount).toBeVisible();

    // 2. Loading state → shows TableSkeleton with aria-busy
    await page.unroute('**/api/firefly/accounts/*/transactions');
    await page.route('**/api/firefly/accounts/*/transactions', async (route) => {
      await new Promise((r) => setTimeout(r, 500));
      await route.fulfill({ json: txs });
    });
    await dashboard.selectAccount('Checking');
    await expect(dashboard.skeleton).toBeVisible();

    // Wait for loading to finish
    await page.waitForResponse(
      (r) => r.url().includes('/api/firefly/accounts/1/transactions'),
    );
    await expect(dashboard.skeleton).toBeHidden();

    // 3. Error state → shows alert with degraded message
    await page.unroute('**/api/firefly/accounts/*/transactions');
    await page.route('**/api/firefly/accounts/*/transactions', (route) =>
      route.fulfill({ status: 500, json: { error: 'Internal Server Error' } }),
    );
    // Navigate to fresh page to reset query state
    await dashboard.goto();
    await dashboard.selectAccount('Checking');
    await expect(dashboard.statusAlert).toBeVisible();

    // 4. Empty results → shows "No transactions in this range"
    await page.unroute('**/api/firefly/accounts/*/transactions');
    await page.route('**/api/firefly/accounts/*/transactions', (route) =>
      route.fulfill({ json: [] }),
    );
    await dashboard.goto();
    await dashboard.selectAccount('Checking');
    await expect(dashboard.browseEmpty).toBeVisible();
  });

  test('R1.6 Date inputs have correct min/max constraints', async ({ page }) => {
    const dashboard = new DashboardPage(page);
    await dashboard.goto();

    // From input max should be end - 1 day
    const toValue = await dashboard.toInput.inputValue();
    const today = toValue; // default end is today
    // Default start is first of month, end is today
    const fromValue = await dashboard.fromInput.inputValue();

    // Verify max attribute on from input
    await expect(dashboard.fromInput).toHaveAttribute('max');

    // Verify min attribute on to input
    await expect(dashboard.toInput).toHaveAttribute('min');

    // The max of from input should be before end date
    const maxAttr = await dashboard.fromInput.getAttribute('max');
    expect(maxAttr).toBeTruthy();
    expect(maxAttr < today || maxAttr === today).toBeTruthy();

    // The min of to input should be after start date
    const minAttr = await dashboard.toInput.getAttribute('min');
    expect(minAttr).toBeTruthy();
    expect(minAttr > fromValue || minAttr === fromValue).toBeTruthy();
  });
});
