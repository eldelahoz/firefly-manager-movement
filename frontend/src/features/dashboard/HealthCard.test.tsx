import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClientProvider } from '@tanstack/react-query';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { HealthCard } from './HealthCard';
import { testQueryClient } from '@/shared/lib/queryClient';

const server = setupServer(http.get('/health', () => HttpResponse.json({ status: 'ok' })));
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

function renderCard() {
  render(
    <QueryClientProvider client={testQueryClient()}>
      <HealthCard />
    </QueryClientProvider>,
  );
}

test('ok state shows version when provided', async () => {
  server.use(
    http.get('/health', () =>
      HttpResponse.json({ status: 'ok', ff3: { reachable: true, version: '6.1.0' } }),
    ),
  );
  renderCard();
  expect(await screen.findByText('Firefly III connected')).toBeInTheDocument();
  expect(screen.getByText('FF3 v6.1.0')).toBeInTheDocument();
});

test('degraded state shows banner with retry and disabled settings', async () => {
  server.use(http.get('/health', () => HttpResponse.json({ status: 'degraded' })));
  renderCard();
  expect(await screen.findByRole('alert')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Retry' })).toBeEnabled();
  expect(screen.getByRole('button', { name: 'Open Settings' })).toBeDisabled();
});
