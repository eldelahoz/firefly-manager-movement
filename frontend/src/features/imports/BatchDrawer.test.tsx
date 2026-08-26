import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { handlers, fixtures } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { BatchDrawer } from './BatchDrawer';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('loads movements and renders Missing pill', async () => {
  renderWithProviders(<BatchDrawer batchId={1} onClose={() => {}} />);
  expect(await screen.findByText('Book')).toBeInTheDocument();
  expect(screen.getAllByText('Missing', { selector: 'span' }).length).toBeGreaterThan(0);
});

test('shows probe hint when no movements', async () => {
  server.use(
    http.get('/api/imports/:id', ({ params }) =>
      HttpResponse.json(
        params.id === '2'
          ? { batch: { ...fixtures.imports[0], id: 2 }, movements: [] }
          : { batch: fixtures.imports[0], movements: fixtures.results.missing },
      ),
    ),
  );
  renderWithProviders(<BatchDrawer batchId={2} onClose={() => {}} />);
  expect(await screen.findByText('no movements (mapping probe)')).toBeInTheDocument();
});

test('backdrop click closes the drawer', async () => {
  let closed = false;
  renderWithProviders(<BatchDrawer batchId={1} onClose={() => (closed = true)} />);
  await screen.findByText('Book');
  await userEvent.click(screen.getByRole('dialog'));
  expect(closed).toBe(true);
});
