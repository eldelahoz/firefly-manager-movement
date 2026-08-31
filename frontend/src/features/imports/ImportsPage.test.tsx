import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { ImportsPage } from './ImportsPage';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

const parsedImports = [
  {
    id: 2,
    filename: 'pending.csv',
    file_hash: 'h2',
    format: 'csv',
    currency: 'EUR',
    status: 'parsed',
    created_at: '2026-08-22T10:00:00Z',
    account_id: null,
    account_name: null,
    movements_count: 3,
  },
  {
    id: 3,
    filename: 'stub.csv',
    file_hash: 'h3',
    format: 'csv',
    currency: 'EUR',
    status: 'parsed',
    created_at: '2026-08-23T10:00:00Z',
    account_id: null,
    account_name: null,
    movements_count: 0,
  },
];

test('renders batch row with status pill', async () => {
  renderWithProviders(<ImportsPage />);
  expect(await screen.findByText('bank.csv')).toBeInTheDocument();
  expect(screen.getByText('Reconciled', { selector: 'span' })).toBeInTheDocument();
});

test('search with no match shows empty state', async () => {
  renderWithProviders(<ImportsPage />);
  await screen.findByText('bank.csv');
  await userEvent.type(screen.getByPlaceholderText('Search by filename'), 'zzz');
  expect(await screen.findByText('No imports yet')).toBeInTheDocument();
});

test('status select filters parsed-only rows out', async () => {
  renderWithProviders(<ImportsPage />);
  await screen.findByText('bank.csv');
  await userEvent.click(screen.getByRole('combobox', { name: 'Filter by status' }));
  await userEvent.click(await screen.findByRole('option', { name: 'Parsed' }));
  expect(await screen.findByText('No imports yet')).toBeInTheDocument();
});

test('date range filters out older rows', async () => {
  renderWithProviders(<ImportsPage />);
  await screen.findByText('bank.csv');
  fireEvent.change(screen.getByLabelText('From'), { target: { value: '2026-08-21' } });
  expect(await screen.findByText('No imports yet')).toBeInTheDocument();
});

test('parsed rows show resume (with movements) and delete actions', async () => {
  server.use(http.get('/api/imports', () => HttpResponse.json(parsedImports)));
  renderWithProviders(<ImportsPage />);
  expect(await screen.findByText('pending.csv')).toBeInTheDocument();

  const resumeButtons = screen.getAllByRole('button', { name: 'Resume' });
  expect(resumeButtons).toHaveLength(1); // only pending.csv (stub has 0 movements)
  expect(screen.getAllByRole('button', { name: 'Delete' })).toHaveLength(2);
  expect(screen.queryByTitle('View results')).not.toBeInTheDocument();
  expect(screen.queryByTitle('Re-run reconciliation')).not.toBeInTheDocument();
});

test('clicking resume opens wizard at the reconcile-config step', async () => {
  server.use(http.get('/api/imports', () => HttpResponse.json(parsedImports)));
  renderWithProviders(<ImportsPage />);
  await screen.findByText('pending.csv');
  await userEvent.click(screen.getByRole('button', { name: 'Resume' }));
  expect(await screen.findByText('Firefly III account')).toBeInTheDocument();
  expect(await screen.findByRole('button', { name: 'Reconcile' })).toBeInTheDocument();
});

test('delete parsed import asks confirmation and removes row', async () => {
  let current = parsedImports;
  let deleted = 0;
  server.use(
    http.get('/api/imports', () => HttpResponse.json(current)),
    http.delete('/api/imports/:id', ({ params }) => {
      deleted += 1;
      current = current.filter((b) => b.id !== Number(params.id));
      return HttpResponse.json({ id: Number(params.id), status: 'deleted' });
    }),
  );
  renderWithProviders(<ImportsPage />);
  await screen.findByText('pending.csv');
  await userEvent.click(screen.getAllByRole('button', { name: 'Delete' })[0]);
  expect(await screen.findByText('Delete import')).toBeInTheDocument();
  expect(screen.getAllByText(/pending.csv/).length).toBeGreaterThanOrEqual(2);
  await userEvent.click(screen.getByRole('button', { name: 'Confirm' }));
  await waitFor(() => expect(deleted).toBe(1));
  expect(await screen.findByText('stub.csv')).toBeInTheDocument();
  expect(screen.queryByText('pending.csv')).not.toBeInTheDocument();
});

test('wizard resets to step 1 on close and reopen', async () => {
  renderWithProviders(<ImportsPage />);
  await screen.findByText('bank.csv');
  await userEvent.click(screen.getByRole('button', { name: 'New import' }));
  expect(await screen.findByText('Drag CSV or OFX here')).toBeInTheDocument();
  await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
  await userEvent.click(screen.getByRole('button', { name: 'New import' }));
  expect(await screen.findByText('Drag CSV or OFX here')).toBeInTheDocument();
});
