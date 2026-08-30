import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { useState } from 'react';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { ReconcileConfigStep } from './ReconcileConfigStep';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

function props(over: Partial<Parameters<typeof ReconcileConfigStep>[0]> = {}) {
  return {
    batchId: 1,
    submitting: false,
    error: null as string | null,
    onStart: () => {},
    onError: () => {},
    onDone: () => {},
    onBack: () => {},
    onCancel: () => {},
    ...over,
  };
}

async function pickAccount(name: string) {
  await userEvent.click(await screen.findByLabelText('Firefly III account'));
  await userEvent.click(await screen.findByRole('option', { name }));
}

test('currency mismatch shows blocking banner and disables finish', async () => {
  renderWithProviders(<ReconcileConfigStep {...props()} />);
  await pickAccount('Savings USD (USD)');
  expect(
    await screen.findByText(
      'Currency mismatch between the batch (EUR) and the selected account (USD)',
    ),
  ).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Reconcile' })).toBeDisabled();
});

test('happy path posts reconcile with computed window and calls onDone', async () => {
  let body: unknown = null;
  server.use(
    http.post('/api/imports/1/reconcile', async ({ request }) => {
      body = await request.json();
      return HttpResponse.json({
        reconciliation_id: 9,
        batch_id: 1,
        tolerance_days: 0,
        window_start: '2026-08-01',
        window_end: '2026-08-10',
        matched_count: 1,
        missing_count: 2,
        extra_count: 1,
        duplicate_count: 1,
      });
    }),
  );
  let done = 0;
  renderWithProviders(<ReconcileConfigStep {...props({ onDone: () => (done += 1) })} />);
  await pickAccount('Checking (EUR)');
  await userEvent.click(screen.getByRole('button', { name: '0' }));
  const finish = await screen.findByRole('button', { name: 'Reconcile' });
  await waitFor(() => expect(finish).toBeEnabled());
  await userEvent.click(finish);
  await waitFor(() => expect(done).toBe(1));
  expect(body).toMatchObject({ ff3_account_id: 1, tolerance_days: 0 });
});

test('initialAccountId preselects the account', async () => {
  renderWithProviders(<ReconcileConfigStep {...props({ initialAccountId: '1' })} />);
  const trigger = await screen.findByLabelText('Firefly III account');
  expect(trigger).toHaveTextContent('Checking (EUR)');
});

function ErrorHarness() {
  const [error, setError] = useState<string | null>(null);
  return <ReconcileConfigStep {...props({ error, onError: setError })} />;
}

test('400 from reconcile surfaces error banner', async () => {
  server.use(
    http.post('/api/imports/1/reconcile', () =>
      HttpResponse.json({ error: 'currency mismatch batch/account' }, { status: 400 }),
    ),
  );
  renderWithProviders(<ErrorHarness />);
  await pickAccount('Checking (EUR)');
  const finish = await screen.findByRole('button', { name: 'Reconcile' });
  await waitFor(() => expect(finish).toBeEnabled());
  await userEvent.click(finish);
  expect(await screen.findByText('currency mismatch batch/account')).toBeInTheDocument();
});
