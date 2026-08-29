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

test('renders description, amount and ignore action', () => {
  renderWithProviders(<MissingRow movement={movement} currency="EUR" batchId={1} />);
  expect(screen.getByText('Book')).toBeInTheDocument();
  expect(screen.getByText('-€99.00')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Ignore' })).toBeInTheDocument();
});

test('failed report pill shows with reason tooltip', () => {
  renderWithProviders(
    <MissingRow
      movement={movement}
      currency="EUR"
      batchId={1}
      report={{ outcome: 'failed', reason: 'FF3 502' }}
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
      batchId={1}
      report={{ outcome: 'already_created' }}
    />,
  );
  expect(screen.getByText('Already created')).toBeInTheDocument();
});
