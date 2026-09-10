import type { Page, Route, Request } from '@playwright/test';
import { fixtures } from '../fixtures/fixtures';

const { accounts, txs } = fixtures;

interface AccountsMockOptions {
  /** Simulate loading delay (ms) for /api/firefly/accounts */
  accountsDelay?: number;
  /** Simulate error for /api/firefly/accounts */
  accountsError?: boolean;
}

/** Filter transactions by date range (open-ended: undefined = no bound). */
export function filterByDate(
  start?: string,
  end?: string,
): typeof txs {
  return txs.filter((tx) => {
    if (start && tx.date < start) return false;
    if (end && tx.date > end) return false;
    return true;
  });
}

function parsePostBody(request: Request): { start?: string; end?: string } {
  try {
    const body = request.postDataJSON() as { start?: string; end?: string } | undefined;
    return body ?? {};
  } catch {
    return {};
  }
}

export async function mockAccounts(
  page: Page,
  opts: AccountsMockOptions = {},
): Promise<void> {
  const { accountsDelay = 0, accountsError = false } = opts;

  await page.route('**/api/firefly/accounts', async (route: Route) => {
    if (accountsDelay > 0) {
      await new Promise((r) => setTimeout(r, accountsDelay));
    }
    if (accountsError) {
      await route.fulfill({ status: 500, json: { error: 'Internal Server Error' } });
      return;
    }
    await route.fulfill({ json: accounts });
  });
}

export async function mockAccountTxs(
  page: Page,
  opts: { delay?: number; error?: boolean; filtered?: boolean } = {},
): Promise<void> {
  const { delay = 0, error = false, filtered = true } = opts;

  await page.route('**/api/firefly/accounts/*/transactions', async (route: Route) => {
    if (delay > 0) {
      await new Promise((r) => setTimeout(r, delay));
    }
    if (error) {
      await route.fulfill({ status: 500, json: { error: 'Internal Server Error' } });
      return;
    }
    if (filtered) {
      const { start, end } = parsePostBody(route.request());
      if (start || end) {
        await route.fulfill({ json: filterByDate(start, end) });
        return;
      }
    }
    await route.fulfill({ json: txs });
  });
}
