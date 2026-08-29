import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { Toaster } from 'sonner';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { ApplyPanel } from './ApplyPanel';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('apply mixed outcomes: toast recap and onDone', async () => {
  let body: unknown = null;
  server.use(
    http.post('/api/imports/1/apply', async ({ request }) => {
      body = await request.json();
      return HttpResponse.json({
        reports: [
          { movement_id: 12, outcome: 'created', ff3_id: 100 },
          { movement_id: 13, outcome: 'already_created' },
        ],
        created_count: 1,
        already_created_count: 1,
        failed_count: 0,
        reconciliation_id: 1,
      });
    }),
  );
  let done = false;
  renderWithProviders(
    <ApplyPanel batchId={1} selected={[12, 13]} onClose={() => {}} onDone={() => (done = true)} />,
  );
  expect(screen.getByRole('button', { name: 'Create in Firefly III' })).toBeDisabled();
  await screen.findByRole('option', { name: 'Checking (EUR)' });
  await userEvent.selectOptions(await screen.findByLabelText('Counterparty account'), '1');
  const btn = screen.getByRole('button', { name: 'Create in Firefly III' });
  await waitFor(() => expect(btn).toBeEnabled());
  await userEvent.click(btn);
  await waitFor(() => expect(done).toBe(true));
  expect(body).toMatchObject({ movement_ids: [12, 13], counterparty_ff3_id: 1 });
});

test('apply failure shows error toast and keeps panel open', async () => {
  server.use(
    http.post('/api/imports/1/apply', () =>
      HttpResponse.json({ error: 'apply failed' }, { status: 500 }),
    ),
  );
  let done = false;
  renderWithProviders(
    <>
      <ApplyPanel batchId={1} selected={[12]} onClose={() => {}} onDone={() => (done = true)} />
      <Toaster />
    </>,
  );
  await screen.findByRole('option', { name: 'Checking (EUR)' });
  await userEvent.selectOptions(await screen.findByLabelText('Counterparty account'), '1');
  const btn = screen.getByRole('button', { name: 'Create in Firefly III' });
  await waitFor(() => expect(btn).toBeEnabled());
  await userEvent.click(btn);
  expect(await screen.findByText('apply failed')).toBeInTheDocument();
  expect(done).toBe(false);
  expect(screen.getByRole('dialog')).toBeInTheDocument();
});
