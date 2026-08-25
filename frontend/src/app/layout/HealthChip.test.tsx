import { afterAll, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { QueryClientProvider } from '@tanstack/react-query';
import { render } from '@testing-library/react';
import { HealthChip } from './HealthChip';
import { testQueryClient } from '@/shared/lib/queryClient';

const server = setupServer(http.get('/health', () => HttpResponse.json({ status: 'ok' })));
beforeAll(() => server.listen());
afterAll(() => server.close());

test('shows ok when healthy', async () => {
  render(
    <QueryClientProvider client={testQueryClient()}>
      <HealthChip />
    </QueryClientProvider>,
  );
  expect(await screen.findByText('Firefly III connected')).toBeInTheDocument();
});
