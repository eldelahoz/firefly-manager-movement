import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import { setupServer } from 'msw/node';
import { handlers, fixtures } from '@/test/handlers';
import type { MovementDTO } from '@/shared/api/dto';
import { renderWithProviders } from '@/test/utils';
import { MissingRow } from './MissingRow';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

const movement = fixtures.results.missing[0] as MovementDTO;
const accounts: never[] = [];
const tags: never[] = [];
const categories: never[] = [];
const budgets: never[] = [];

test('renders grid columns with status pill and actions', () => {
  renderWithProviders(
    <MissingRow
      movement={movement}
      currency="EUR"
      accounts={accounts}
      tags={tags}
      categories={categories}
      budgets={budgets}
      defaultTag={null}
      onSave={() => {}}
      onCancel={() => {}}
    />,
  );
  expect(screen.getByText('Missing')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Edit' })).toBeInTheDocument();
});

test('renders description, amount and edit action', () => {
  renderWithProviders(
    <MissingRow
      movement={movement}
      currency="EUR"
      accounts={accounts}
      tags={tags}
      categories={categories}
      budgets={budgets}
      defaultTag={null}
      onSave={() => {}}
      onCancel={() => {}}
    />,
  );
  expect(screen.getByText('Book')).toBeInTheDocument();
  expect(screen.getByText('-€99.00')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Edit' })).toBeInTheDocument();
});

test('failed report pill shows with reason tooltip', () => {
  renderWithProviders(
    <MissingRow
      movement={movement}
      currency="EUR"
      report={{ outcome: 'failed', reason: 'FF3 502' }}
      accounts={accounts}
      tags={tags}
      categories={categories}
      budgets={budgets}
      defaultTag={null}
      onSave={() => {}}
      onCancel={() => {}}
    />,
  );
  expect(screen.getByText('Failed')).toBeInTheDocument();
  expect(screen.getByTitle('FF3 502')).toBeInTheDocument();
});

test('already_created shows neutral pill', () => {
  renderWithProviders(
    <MissingRow
      movement={movement}
      currency="EUR"
      report={{ outcome: 'already_created' }}
      accounts={accounts}
      tags={tags}
      categories={categories}
      budgets={budgets}
      defaultTag={null}
      onSave={() => {}}
      onCancel={() => {}}
    />,
  );
  expect(screen.getByText('Already created')).toBeInTheDocument();
});
