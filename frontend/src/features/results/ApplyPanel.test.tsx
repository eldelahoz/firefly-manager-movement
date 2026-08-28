import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
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
  renderWithProviders(<ApplyPanel batchId={1} selected={[12, 13]} onDone={() => (done = true)} />);
  await screen.findByRole('option', { name: 'Checking (EUR)' });
  await userEvent.selectOptions(await screen.findByLabelText('Counterparty account'), '1');
  const btn = screen.getByRole('button', { name: 'Create in Firefly III' });
  await waitFor(() => expect(btn).toBeEnabled());
  await userEvent.click(btn);
  await waitFor(() => expect(done).toBe(true));
  expect(body).toMatchObject({ movement_ids: [12, 13], counterparty_ff3_id: 1 });
});
