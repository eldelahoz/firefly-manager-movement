import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { Toaster } from 'sonner';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { DeleteDuplicateDialog } from './DeleteDuplicateDialog';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('404 on delete shows explanatory toast', async () => {
  server.use(
    http.delete('/api/firefly/transactions/:id', () =>
      HttpResponse.json({ error: 'not in run cache' }, { status: 404 }),
    ),
  );
  renderWithProviders(
    <>
      <DeleteDuplicateDialog
        tx={{ id: 42, date: '2026-08-01', amount_cents: -4200, description: 'Coffee' }}
        currency="EUR"
        onClose={() => {}}
      />
      <Toaster />
    </>,
  );
  await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
  await userEvent.click(screen.getByRole('button', { name: 'Confirm' }));
  expect(
    await screen.findByText('Only transactions seen in a reconciliation can be deleted from here'),
  ).toBeInTheDocument();
});

test('non-404 error shows generic error toast', async () => {
  server.use(
    http.delete('/api/firefly/transactions/:id', () =>
      HttpResponse.json({ error: 'FF3 unreachable' }, { status: 502 }),
    ),
  );
  renderWithProviders(
    <>
      <DeleteDuplicateDialog
        tx={{ id: 42, date: '2026-08-01', amount_cents: -4200, description: 'Coffee' }}
        currency="EUR"
        onClose={() => {}}
      />
      <Toaster />
    </>,
  );
  await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
  await userEvent.click(screen.getByRole('button', { name: 'Confirm' }));
  expect(await screen.findByText('Something went wrong')).toBeInTheDocument();
});

test('success closes the dialog', async () => {
  let closed = false;
  renderWithProviders(
    <DeleteDuplicateDialog
      tx={{ id: 42, date: '2026-08-01', amount_cents: -4200, description: 'Coffee' }}
      currency="EUR"
      onClose={() => {
        closed = true;
      }}
    />,
  );
  await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
  await userEvent.click(screen.getByRole('button', { name: 'Confirm' }));
  await waitFor(() => expect(closed).toBe(true));
});
