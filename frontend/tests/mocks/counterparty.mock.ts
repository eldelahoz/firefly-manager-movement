import type { Page } from '@playwright/test';
import { fixtures } from '../fixtures/fixtures';

// Counterparty-accounts feature (RF-08 v0.10): fake accounts per the team-brief contract.
// The batch account is Checking (id 1, EUR). Savings USD (id 2) is another currency.
const acct = (id: number, name: string, type: string, extra: Record<string, unknown> = {}) => ({
  id,
  name,
  type,
  currency: 'EUR',
  iban: null,
  account_number: null,
  role: null,
  liability_type: null,
  ...extra,
});

export const cp = {
  batchAccountId: 1,
  asset: [
    acct(1, 'Checking', 'asset', { role: 'defaultAsset' }),
    acct(2, 'Savings USD', 'asset', { role: 'savingAsset', currency: 'USD' }),
    acct(3, 'Rainy Day', 'asset', { role: 'savingAsset' }),
    acct(4, 'Visa Card', 'asset', { role: 'ccAsset' }),
  ],
  expense: [acct(51, 'Supermarket', 'expense'), acct(52, 'Pizza Place', 'expense')],
  revenue: [acct(61, 'Employer', 'revenue'), acct(62, 'Refunds Inc', 'revenue')],
  liability: [
    acct(71, 'Car Loan', 'liabilities', { liability_type: 'loan' }),
    acct(72, 'House Mortgage', 'liabilities', { liability_type: 'mortgage' }),
  ],
};

/** Reconciled batch whose target account is Checking, so the picker must leave it out. */
export const cpBatch = {
  ...fixtures.imports[0],
  account_id: cp.batchAccountId,
  account_name: 'Checking',
};

/** Overrides the default accounts mock (registered later = wins): serves every `?type=`. */
export async function mockCounterpartyAccounts(page: Page): Promise<void> {
  await page.route('**/api/firefly/accounts?*', async (route) => {
    if (route.request().url().includes('/transactions')) return route.fallback();
    const type = new URL(route.request().url()).searchParams.get('type');
    const json =
      type && type in cp ? cp[type as 'asset' | 'expense' | 'revenue' | 'liability'] : [];
    await route.fulfill({ json });
  });
  // Plain listing = asset accounts (what the batch target select uses).
  await page.route('**/api/firefly/accounts', async (route) => {
    await route.fulfill({ json: cp.asset });
  });
}

type Outcome = 'created' | 'failed' | 'already_created';
export type CpApplyBody = { movements: Record<string, unknown>[] };

/**
 * Results page with the counterparty fixtures. `reports` maps a movement id to the apply report
 * the fake backend answers with (unlisted ids get no report). Returns the captured apply bodies.
 */
export async function mockCounterpartyResults(
  page: Page,
  missing: Record<string, unknown>[],
  reports: Record<
    number,
    { outcome: Outcome; reason?: string; transaction_type?: string | null }
  > = {},
) {
  await mockCounterpartyAccounts(page);
  await page.route('**/api/imports/1', (r) =>
    r.fulfill({
      json: { batch: cpBatch, movements: [], headers: [], detected_delimiter: ',' },
    }),
  );
  await page.route('**/api/imports/1/results', (r) =>
    r.fulfill({
      json: { reconciliation_id: 1, matched: [], missing, ignored: [], extra: [] },
    }),
  );
  const applied: CpApplyBody[] = [];
  await page.route('**/api/imports/1/apply', async (r) => {
    const body = r.request().postDataJSON() as CpApplyBody;
    applied.push(body);
    const list = body.movements.flatMap((m) => {
      const rep = reports[m.movement_id as number];
      return rep ? [{ movement_id: m.movement_id, ...rep }] : [];
    });
    const count = (o: Outcome) => list.filter((x) => x.outcome === o).length;
    await r.fulfill({
      json: {
        reports: list,
        created_count: count('created'),
        already_created_count: count('already_created'),
        failed_count: count('failed'),
      },
    });
  });
  return applied;
}
