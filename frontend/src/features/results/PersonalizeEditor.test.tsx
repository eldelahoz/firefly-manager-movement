import { expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/test/utils';
import { PersonalizeEditor } from './PersonalizeEditor';
import type { Customization } from './customization';

const accounts = [
  { id: 1, name: 'Checking', currency: 'EUR', type: 'asset' },
  { id: 2, name: 'Groceries', currency: 'EUR', type: 'expense' },
  { id: 3, name: 'Salary', currency: 'EUR', type: 'revenue' },
];

test('tags pre-filled with default creation tag', () => {
  renderWithProviders(
    <PersonalizeEditor
      initial={{}}
      defaultTag="ffm-import"
      direction="withdrawal"
      accounts={accounts}
      tags={['food', 'transport']}
      categories={['Food', 'Bills']}
      budgets={['Monthly']}
      onSave={() => {}}
      onCancel={() => {}}
      submitLabel="Save"
    />,
  );
  expect(screen.getByRole('button', { name: 'ffm-import' })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
});

test('unchecking default tag and saving yields tags: []', async () => {
  let saved: Customization | null = null;
  renderWithProviders(
    <PersonalizeEditor
      initial={{}}
      defaultTag="ffm-import"
      direction="withdrawal"
      accounts={accounts}
      tags={['ffm-import']}
      categories={[]}
      budgets={[]}
      onSave={(c) => (saved = c)}
      onCancel={() => {}}
      submitLabel="Save"
    />,
  );
  await userEvent.click(screen.getByRole('button', { name: 'ffm-import' }));
  expect(screen.getByRole('button', { name: 'ffm-import' })).toHaveAttribute(
    'aria-pressed',
    'false',
  );
  await userEvent.click(screen.getByRole('button', { name: 'Save' }));
  expect(saved).toBeDefined();
  expect(saved!.tags).toEqual([]);
});

test('withdrawal direction lists expense and asset accounts', async () => {
  renderWithProviders(
    <PersonalizeEditor
      initial={{}}
      defaultTag={null}
      direction="withdrawal"
      accounts={accounts}
      tags={[]}
      categories={[]}
      budgets={[]}
      onSave={() => {}}
      onCancel={() => {}}
      submitLabel="Save"
    />,
  );
  await userEvent.click(screen.getByLabelText('Counterparty account'));
  expect(screen.getByRole('option', { name: /Checking/ })).toBeInTheDocument();
  expect(screen.getByRole('option', { name: /Groceries/ })).toBeInTheDocument();
  expect(screen.queryByRole('option', { name: /Salary/ })).not.toBeInTheDocument();
});

test('reports dirty state when a field changes', async () => {
  const dirtyStates: boolean[] = [];
  renderWithProviders(
    <PersonalizeEditor
      initial={{}}
      defaultTag={null}
      direction="withdrawal"
      accounts={accounts}
      tags={[]}
      categories={[]}
      budgets={[]}
      onDirtyChange={(d) => dirtyStates.push(d)}
      onSave={() => {}}
      onCancel={() => {}}
      submitLabel="Save"
    />,
  );
  await userEvent.type(screen.getByLabelText('Description'), 'x');
  expect(dirtyStates).toEqual([true]);
});
