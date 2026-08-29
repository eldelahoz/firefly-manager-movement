import type { ReactElement } from 'react';
import { render } from '@testing-library/react';
import { QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import { testQueryClient } from '@/shared/lib/queryClient';
import { PageHeaderProvider } from '@/shared/ui/PageHeader';

export function renderWithProviders(ui: ReactElement, { route = '/' } = {}) {
  return render(
    <QueryClientProvider client={testQueryClient()}>
      <MemoryRouter initialEntries={[route]}>
        <PageHeaderProvider>{ui}</PageHeaderProvider>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}
