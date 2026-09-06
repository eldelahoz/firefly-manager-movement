import { expect, test, vi } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { fixtures } from '@/test/handlers';
import type { MovementDTO } from '@/shared/api/dto';
import { renderWithProviders } from '@/test/utils';
import type { Customization } from './customization';
import { EditMovementDialog } from './EditMovementDialog';

const movement = fixtures.results.missing[0] as MovementDTO;

function renderDialog(
  overrides: {
    initial?: Customization;
    onSave?: (c: Customization) => void;
    onClose?: () => void;
  } = {},
) {
  return renderWithProviders(
    <EditMovementDialog
      movement={movement}
      initial={overrides.initial ?? {}}
      defaultTag={null}
      currency="EUR"
      accounts={[]}
      tags={[]}
      categories={[]}
      budgets={[]}
      onSave={overrides.onSave ?? (() => {})}
      onClose={overrides.onClose ?? (() => {})}
    />,
  );
}

test('renders title, movement summary and all editor fields', () => {
  renderDialog();
  expect(screen.getByText('Edit movement')).toBeInTheDocument();
  expect(screen.getByLabelText('Counterparty account')).toBeInTheDocument();
  expect(screen.getByLabelText('Description')).toBeInTheDocument();
  expect(screen.getByText('Tags')).toBeInTheDocument();
  expect(screen.getByLabelText('Category (optional)')).toBeInTheDocument();
  expect(screen.getByLabelText('Budget (optional)')).toBeInTheDocument();
  expect(screen.getByLabelText('Notes (optional)')).toBeInTheDocument();
  expect(screen.getByText('Attachment')).toBeInTheDocument();
});

test('initial customization prefills fields', () => {
  renderDialog({ initial: { description: 'prev' } });
  expect(screen.getByLabelText('Description')).toHaveValue('prev');
});

test('save button returns customization', async () => {
  const onSave = vi.fn();
  renderDialog({ onSave });
  await userEvent.type(screen.getByLabelText('Description'), 'coffee');
  await userEvent.click(screen.getByRole('button', { name: 'Save' }));
  expect(onSave).toHaveBeenCalledTimes(1);
  expect(onSave.mock.calls[0][0].description).toBe('coffee');
});

test('cancel without changes closes immediately', async () => {
  const onClose = vi.fn();
  renderDialog({ onClose });
  await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
  expect(onClose).toHaveBeenCalledTimes(1);
});

test('close with unsaved changes asks confirmation: keep editing or discard', async () => {
  const onClose = vi.fn();
  renderDialog({ onClose });
  await userEvent.type(screen.getByLabelText('Description'), 'x');
  await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
  expect(screen.getByText('Discard changes?')).toBeInTheDocument();
  expect(onClose).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole('button', { name: 'Keep editing' }));
  expect(screen.queryByText('Discard changes?')).not.toBeInTheDocument();
  expect(onClose).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
  await userEvent.click(screen.getByRole('button', { name: 'Discard' }));
  expect(onClose).toHaveBeenCalledTimes(1);
});
