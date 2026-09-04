import { expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/test/utils';
import { ConfirmCreateDialog } from './ConfirmCreateDialog';
import type { Customization } from './customization';

const accounts = [
  { id: 1, name: 'Checking', currency: 'EUR', type: 'asset' },
  { id: 2, name: 'Savings', currency: 'EUR', type: 'asset' },
];

test('renders per-movement summary and origin tag note', () => {
  const movements = [
    {
      id: 10,
      description: 'Coffee',
      amountCents: -450,
      customization: { counterpartyId: 1, tags: ['food'] } satisfies Customization,
    },
  ];
  renderWithProviders(
    <ConfirmCreateDialog
      open
      batchId={1}
      filename="bank.csv"
      movements={movements}
      accounts={accounts}
      onConfirm={() => {}}
      onClose={() => {}}
    />,
  );
  expect(screen.getByText('Coffee')).toBeInTheDocument();
  expect(screen.getByText(/ffm:bank\.csv:1/)).toBeInTheDocument();
  expect(screen.getByText(/Checking/)).toBeInTheDocument();
  expect(screen.getByText(/food/)).toBeInTheDocument();
});

test('confirm enabled without counterparty (optional field)', () => {
  const movements = [{ id: 10, description: 'Coffee', amountCents: -4200, customization: {} }];
  renderWithProviders(
    <ConfirmCreateDialog
      open
      batchId={1}
      filename="bank.csv"
      movements={movements}
      accounts={accounts}
      onConfirm={() => {}}
      onClose={() => {}}
    />,
  );
  expect(screen.getByRole('button', { name: 'Create' })).toBeEnabled();
  expect(screen.queryByText('Missing counterparty account')).not.toBeInTheDocument();
});

test('confirm enabled after personalization and click calls onConfirm', async () => {
  let confirmed = false;
  const movements = [
    {
      id: 10,
      description: 'Coffee',
      amountCents: -450,
      customization: { counterpartyId: 1 } satisfies Customization,
    },
  ];
  renderWithProviders(
    <ConfirmCreateDialog
      open
      batchId={1}
      filename="bank.csv"
      movements={movements}
      accounts={accounts}
      onConfirm={() => (confirmed = true)}
      onClose={() => {}}
    />,
  );
  expect(screen.getByRole('button', { name: 'Create' })).toBeEnabled();
  await userEvent.click(screen.getByRole('button', { name: 'Create' }));
  expect(confirmed).toBe(true);
});
