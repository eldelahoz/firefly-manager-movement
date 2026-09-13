import type { Page, Locator } from '@playwright/test';

export class DashboardPage {
  readonly page: Page;
  readonly accountCombobox: Locator;
  readonly fromInput: Locator;
  readonly toInput: Locator;
  readonly browseList: Locator;
  readonly browseEmpty: Locator;
  readonly browseSelectAccount: Locator;
  readonly statusAlert: Locator;
  readonly skeleton: Locator;
  readonly loader: Locator;

  constructor(page: Page) {
    this.page = page;
    this.accountCombobox = page.getByRole('combobox', { name: 'Firefly III account' });
    // Use specific role to avoid matching theme toggle button
    this.fromInput = page.getByRole('textbox', { name: 'From' });
    this.toInput = page.getByRole('textbox', { name: 'To' });
    this.browseList = page.getByRole('list');
    this.browseEmpty = page.getByText('No transactions in this range');
    // The paragraph "Select an account" appears below the date inputs; .last() avoids the Select placeholder
    this.browseSelectAccount = page.getByText('Select an account').last();
    this.statusAlert = page.getByRole('alert');
    this.skeleton = page.locator('[aria-busy="true"]');
    this.loader = page.getByRole('status').filter({ hasText: 'Loading' });
  }

  async goto() {
    await this.page.goto('/');
  }

  /** Select an account by name from the Radix Select combobox. */
  async selectAccount(name: string) {
    await this.accountCombobox.click();
    await this.page.getByRole('option', { name }).click();
    // Wait for the Radix Select portal dropdown to close
    await this.page.locator('[role="listbox"]').waitFor({ state: 'hidden', timeout: 3000 });
  }

  /** Set the "From" date input value. */
  async setFromDate(date: string) {
    await this.fromInput.fill(date);
  }

  /** Set the "To" date input value. */
  async setToDate(date: string) {
    await this.toInput.fill(date);
  }

  /** Get all visible transaction rows. */
  async getTransactionRows(): Promise<Locator[]> {
    const items = this.browseList.getByRole('listitem');
    const count = await items.count();
    return Array.from({ length: count }, (_, i) => items.nth(i));
  }

  /** Get transaction text content from a row. */
  async getTransactionText(row: Locator): Promise<string> {
    return (await row.textContent()) ?? '';
  }
}
