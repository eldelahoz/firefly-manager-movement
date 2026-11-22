import { QueryCache, QueryClient } from '@tanstack/react-query';
import { toast } from 'sonner';
import { ApiError } from '@/shared/api/client';
import type { HealthResponse } from '@/shared/api/dto';
import { queryKeys } from './keys';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: 1, staleTime: 30_000, refetchOnWindowFocus: false },
  },
  queryCache: new QueryCache({
    // A 502 is "Firefly III unreachable" (backend BAD_GATEWAY): while health is degraded the banner
    // and sidebar chip already say so. Every other query error still toasts.
    onError: (error) => {
      const ff3Down =
        error instanceof ApiError &&
        error.status === 502 &&
        queryClient.getQueryData<HealthResponse>(queryKeys.health)?.status === 'degraded';
      if (!ff3Down) toast.error(error.message);
    },
  }),
});

/** Failed and still has no data. Without data, a refetch puts the query back in `pending`, so
 *  `isError` alone would unmount the error and its focused Retry on every retry. */
export const failedWithoutData = (q: { data: unknown; errorUpdateCount: number }) =>
  q.data === undefined && q.errorUpdateCount > 0;
