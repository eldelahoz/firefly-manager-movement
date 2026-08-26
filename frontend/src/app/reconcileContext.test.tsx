import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { ReconcileProvider, useRerun } from './reconcileContext';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

function Harness() {
  const open = useRerun();
  return <button onClick={() => open(1)}>open-rerun</button>;
}

test('rerun dialog mounts inside the router without throwing (useNavigate context)', async () => {
  renderWithProviders(
    <ReconcileProvider>
      <Harness />
    </ReconcileProvider>,
  );
  await userEvent.click(screen.getByRole('button', { name: 'open-rerun' }));
  expect(await screen.findByRole('dialog', { name: 'Re-run reconciliation' })).toBeInTheDocument();
});
