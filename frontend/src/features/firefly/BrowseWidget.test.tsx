import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { BrowseWidget } from './BrowseWidget';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

async function pickCheckingAccount() {
  await userEvent.click(await screen.findByLabelText('Firefly III account'));
  await userEvent.click(await screen.findByRole('option', { name: 'Checking' }));
}

test('idle state prompts account selection', async () => {
  renderWithProviders(<BrowseWidget />);
  expect((await screen.findAllByText('Select an account')).length).toBeGreaterThanOrEqual(1);
});

test('populated state lists transactions with status pills', async () => {
  renderWithProviders(<BrowseWidget />);
  await pickCheckingAccount();
  expect(await screen.findByText('Coffee')).toBeInTheDocument();
  expect(screen.getByText('Duplicate A')).toBeInTheDocument();
  expect(screen.getByText('Salary')).toBeInTheDocument();
  expect(screen.getByText('Matched', { selector: 'span' })).toBeInTheDocument();
  expect(screen.getByText('Duplicate', { selector: 'span' })).toBeInTheDocument();
  expect(screen.queryByText('Extra', { selector: 'span' })).not.toBeInTheDocument();
});

test('delete duplicate action only appears on duplicates', async () => {
  renderWithProviders(<BrowseWidget />);
  await pickCheckingAccount();
  await screen.findByText('Coffee');
  const deleteButtons = screen.getAllByRole('button', { name: 'Delete duplicate' });
  expect(deleteButtons).toHaveLength(1);
  const dupRow = screen.getByText('Duplicate A').closest('li');
  expect(dupRow).toContainElement(deleteButtons[0]);
});

test('502 shows error alert', async () => {
  server.use(
    http.post('/api/firefly/accounts/:id/transactions', () =>
      HttpResponse.json({ error: 'ff3 down' }, { status: 502 }),
    ),
  );
  renderWithProviders(<BrowseWidget />);
  await pickCheckingAccount();
  expect(await screen.findByRole('alert')).toBeInTheDocument();
});

test('delete duplicate happy path reaches confirm and DELETE', async () => {
  let deleted = false;
  server.use(
    http.delete('/api/firefly/transactions/:id', () => {
      deleted = true;
      return HttpResponse.json({ ff3_id: '42', status: 'deleted' });
    }),
  );
  renderWithProviders(<BrowseWidget />);
  await pickCheckingAccount();
  await userEvent.click((await screen.findAllByText('Delete duplicate'))[0]);
  expect(await screen.findByRole('dialog')).toBeInTheDocument();
  await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
  await userEvent.click(screen.getByRole('button', { name: 'Confirm' }));
  expect(deleted).toBe(true);
});
