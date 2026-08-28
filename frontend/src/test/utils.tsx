import type { ReactElement } from 'react';
import { render } from '@testing-library/react';
import { QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter, Routes, Route } from 'react-router-dom';
import { testQueryClient } from '@/shared/lib/queryClient';

export function renderWithProviders(ui: ReactElement, { route = '/' } = {}) {
  const pattern = route.replace(/\/(\d+)(?=\/|$)/, '/:batchId');
  return render(
    <QueryClientProvider client={testQueryClient()}>
      <MemoryRouter initialEntries={[route]}>
        <Routes>
          <Route path={pattern} element={ui} />
          <Route path="*" element={ui} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}
