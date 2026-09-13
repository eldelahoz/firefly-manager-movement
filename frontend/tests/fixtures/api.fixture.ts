import { test as base } from '@playwright/test';
import { mockAccounts, mockAccountTxs } from '../mocks';
import { mockSettings, mockVerifySettings, mockTags } from '../mocks';
import { mockHealth } from '../mocks';
import { mockImports, type MockImportsOptions } from '../mocks/imports.mock';

type MockedApiFixtures = {
  /** Mock all endpoints needed for Dashboard page */
  mockDashboard: (opts?: { accountsDelay?: number; txsError?: boolean }) => Promise<void>;
  /** Mock all endpoints needed for Settings page */
  mockSettingsPage: () => Promise<void>;
  /** Mock all endpoints needed for Imports page */
  mockImportsPage: (opts?: MockImportsOptions) => Promise<void>;
};

export const test = base.extend<MockedApiFixtures>({
  mockDashboard: async ({ page }, use) => {
    const setup = async (opts?: { accountsDelay?: number; txsError?: boolean }) => {
      await mockHealth(page);
      await mockAccounts(page, { accountsDelay: opts?.accountsDelay });
      await mockAccountTxs(page, { error: opts?.txsError, filtered: true });
      await mockImports(page);
    };
    await use(setup);
  },

  mockSettingsPage: async ({ page }, use) => {
    const setup = async () => {
      await mockHealth(page);
      await mockSettings(page);
      await mockVerifySettings(page);
      await mockTags(page);
    };
    await use(setup);
  },

  mockImportsPage: async ({ page }, use) => {
    const setup = async (opts?: MockImportsOptions) => {
      await mockHealth(page);
      await mockAccounts(page);
      await mockImports(page, opts);
    };
    await use(setup);
  },
});

export { expect } from '@playwright/test';
