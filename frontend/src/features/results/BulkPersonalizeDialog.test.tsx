import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { BulkPersonalizeDialog } from './BulkPersonalizeDialog';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('fill-only: category applied, other fields omitted', async () => {
  let applied: unknown = null;
  renderWithProviders(
    <BulkPersonalizeDialog
      count={3}
      defaultTag={null}
      onApply={(f) => (applied = f)}
      onClose={() => {}}
    />,
  );
  expect(screen.getByRole('dialog')).toBeInTheDocument();
  await userEvent.click(await screen.findByLabelText('Category (optional)'));
  await userEvent.click(await screen.findByRole('option', { name: 'Groceries' }));
  await userEvent.click(screen.getByRole('button', { name: 'Apply' }));
  await waitFor(() => expect(applied).toEqual({ category: 'Groceries' }));
});

test('tags untouched when chips not toggled even though pre-filled', async () => {
  let applied: unknown = null;
  renderWithProviders(
    <BulkPersonalizeDialog
      count={2}
      defaultTag="ffm-import"
      onApply={(f) => (applied = f)}
      onClose={() => {}}
    />,
  );
  await userEvent.click(screen.getByRole('button', { name: 'Apply' }));
  await waitFor(() => expect(applied).toEqual({}));
});

test('counterparty applied when selected', async () => {
  let applied: unknown = null;
  renderWithProviders(
    <BulkPersonalizeDialog
      count={1}
      defaultTag={null}
      onApply={(f) => (applied = f)}
      onClose={() => {}}
    />,
  );
  await userEvent.click(await screen.findByLabelText('Counterparty account'));
  await userEvent.click(await screen.findByRole('option', { name: 'Checking (EUR)' }));
  await userEvent.click(screen.getByRole('button', { name: 'Apply' }));
  await waitFor(() => expect(applied).toEqual({ counterpartyId: 1 }));
});
