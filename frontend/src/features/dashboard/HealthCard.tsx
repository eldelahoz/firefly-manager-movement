import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getHealth } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';
import { Skeleton } from '@/shared/ui/Skeleton';

export function HealthCard() {
  const { t } = useTranslation();
  const { data, isError, refetch, isPending } = useQuery({
    queryKey: queryKeys.health,
    queryFn: getHealth,
    refetchInterval: 60_000,
  });
  if (isPending) return <Skeleton className="h-28 w-full" />;
  if (isError || data.status === 'degraded') {
    return (
      <div
        role="alert"
        title={t('health.probeHint')}
        className="flex flex-wrap items-center justify-between gap-4 rounded-xl border border-status-missing bg-status-missing-bg p-4 text-status-missing-txt"
      >
        <span>{t('health.degraded')}</span>
        <span className="flex gap-2">
          <button
            onClick={() => void refetch()}
            className="rounded-lg bg-surface px-3 py-2 text-sm font-medium text-text-1"
          >
            {t('common.retry')}
          </button>
          <button disabled className="rounded-lg bg-surface px-3 py-2 text-sm text-text-3">
            {t('common.openSettings')}
          </button>
        </span>
      </div>
    );
  }
  return (
    <div className="flex items-center gap-3 rounded-xl border border-border-subtle bg-surface p-4 shadow-sm">
      <span className="size-3 rounded-full bg-status-matched" />
      <span className="font-medium">{t('health.ok')}</span>
      {data.ff3?.version && <span className="text-text-3">FF3 v{data.ff3.version}</span>}
    </div>
  );
}
