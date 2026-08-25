import { afterAll, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { RecentBatches } from './RecentBatches';
import { useLocation } from 'react-router-dom';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterAll(() => server.close());

test('renders batch filename with status pill', async () => {
  renderWithProviders(<RecentBatches />);
  expect(await screen.findByText('bank.csv')).toBeInTheDocument();
  expect(screen.getByText('Reconciled')).toBeInTheDocument();
});

test('clicking reconciled batch navigates to results', async () => {
  let path = '';
  function Probe() {
    path = useLocation().pathname;
    return null;
  }
  renderWithProviders(
    <>
      <RecentBatches />
      <Probe />
    </>,
  );
  await userEvent.click(await screen.findByText('bank.csv'));
  expect(path).toBe('/results/1');
});
