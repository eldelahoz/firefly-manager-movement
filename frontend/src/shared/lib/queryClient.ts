import { QueryCache, QueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';

export const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: 1, staleTime: 30_000 } },
  queryCache: new QueryCache({
    onError: (error) => toast.error(error.message),
  }),
});

export function testQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: { queries: { retry: false } },
    queryCache: new QueryCache({}),
  });
}
