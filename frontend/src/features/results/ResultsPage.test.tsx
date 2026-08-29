import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { fixtures, handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { Route, Routes } from 'react-router-dom';
import { ResultsPage } from './ResultsPage';

const server = setupServer(...handlers);

const originalRect = HTMLElement.prototype.getBoundingClientRect;
const originalOffsetHeight = Object.getOwnPropertyDescriptor(HTMLElement.prototype, 'offsetHeight');
beforeAll(() => {
  server.listen();
  HTMLElement.prototype.getBoundingClientRect = () =>
    ({
      width: 800,
      height: 600,
      top: 0,
      left: 0,
      right: 800,
      bottom: 600,
      x: 0,
      y: 0,
      toJSON: () => {},
    }) as DOMRect;
  Object.defineProperty(HTMLElement.prototype, 'offsetHeight', {
    configurable: true,
    get: () => 600,
  });
});
afterEach(() => server.resetHandlers());
afterAll(() => {
  server.close();
  HTMLElement.prototype.getBoundingClientRect = originalRect;
  if (originalOffsetHeight)
    Object.defineProperty(HTMLElement.prototype, 'offsetHeight', originalOffsetHeight);
});

function renderResults() {
  return renderWithProviders(
    <Routes>
      <Route path="/results/:batchId" element={<ResultsPage />} />
    </Routes>,
    { route: '/results/1' },
  );
}

test('renders tabs with counts and matched pairs', async () => {
  renderResults();
  expect(await screen.findByText('Matched (1)')).toBeInTheDocument();
  expect(screen.getByText('Missing (2)')).toBeInTheDocument();
  expect(screen.getByText('Extra (1)')).toBeInTheDocument();
  expect(screen.getAllByText('Coffee').length).toBe(2);
  expect(screen.getByRole('button', { name: 'Re-run reconciliation' })).toBeInTheDocument();
});

test('404 without reconciliation shows empty state with CTA', async () => {
  server.use(
    http.get('/api/imports/:id/results', () =>
      HttpResponse.json({ error: 'no reconciliation' }, { status: 404 }),
    ),
  );
  renderResults();
  expect(await screen.findByText('This batch has no reconciliation yet')).toBeInTheDocument();
  expect(screen.getByText('Start an import')).toBeInTheDocument();
});

test('switching to extra shows duplicate with delete action', async () => {
  renderResults();
  await screen.findByText('Matched (1)');
  await userEvent.click(screen.getByRole('tab', { name: 'Extra (1)' }));
  expect(await screen.findByText('Duplicate A')).toBeInTheDocument();
  expect(screen.getByText('Duplicate')).toBeInTheDocument();
  expect(screen.getByText('Delete duplicate')).toBeInTheDocument();
});

test('missing tab supports select-all and opens apply panel', async () => {
  renderResults();
  await screen.findByText('Matched (1)');
  await userEvent.click(screen.getByRole('tab', { name: 'Missing (2)' }));
  const checkboxes = await screen.findAllByRole('checkbox');
  const selectAll = checkboxes[0];
  await userEvent.click(selectAll);
  expect(screen.getByRole('button', { name: 'Create in Firefly III (2)' })).toBeEnabled();
  await userEvent.click(screen.getByRole('button', { name: 'Create in Firefly III (2)' }));
  expect(await screen.findByRole('dialog')).toBeInTheDocument();
  await userEvent.click(screen.getByRole('button', { name: 'Cancel' }));
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Create in Firefly III (2)' })).toBeEnabled();
});

test('virtualizes 1000 matched rows without rendering all', async () => {
  const big = {
    ...fixtures.results,
    matched: Array.from({ length: 1000 }, (_, i) => ({
      movement: {
        id: i,
        line_no: i,
        date: '2026-08-01',
        amount_cents: -100,
        description: `m${i}`,
        status: 'matched' as const,
      },
      ff3: {
        id: i,
        ff3_id: i,
        date: '2026-08-01',
        amount_cents: -100,
        description: `f${i}`,
        is_duplicate: false,
        created_at: '2026-08-01T00:00:00Z',
      },
    })),
  };
  server.use(http.get('/api/imports/:id/results', () => HttpResponse.json(big)));
  renderResults();
  await screen.findByText('Matched (1000)');
  expect(screen.queryByText('m999')).not.toBeInTheDocument();
  expect(document.querySelectorAll('[data-index]').length).toBeLessThan(1000);
});
