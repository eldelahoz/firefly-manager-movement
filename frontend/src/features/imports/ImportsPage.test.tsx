import { afterAll, beforeAll, expect, test } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { ImportsPage } from './ImportsPage';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterAll(() => server.close());

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
