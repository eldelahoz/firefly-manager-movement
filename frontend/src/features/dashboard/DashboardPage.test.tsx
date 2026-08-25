import { afterAll, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import { setupServer } from 'msw/node';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { DashboardPage } from './DashboardPage';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterAll(() => server.close());

test('renders kpis from fixtures', async () => {
  renderWithProviders(<DashboardPage />);
  expect(await screen.findByText('Dashboard')).toBeInTheDocument();
  expect(await screen.findByText('2')).toBeInTheDocument();
  expect(screen.getAllByText('1').length).toBeGreaterThanOrEqual(2);
});
