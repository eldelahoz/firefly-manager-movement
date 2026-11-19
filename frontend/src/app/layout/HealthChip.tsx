import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getHealth } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';

export function HealthChip() {
  const { t } = useTranslation();
  const { data, isError } = useQuery({
    queryKey: queryKeys.health,
    queryFn: getHealth,
    refetchInterval: 60_000,
  });
  const ok = !isError && data?.ff3?.reachable === true;
  // No ff3 block = backend did not probe Firefly III, so the state is unknown, not connected.
  const unknown = isError || !data || (data.status === 'ok' && !data.ff3);
  return (
    <span
      title={t('health.probeHint')}
      className="flex items-center gap-2 rounded-full border border-border-subtle bg-canvas px-3 py-1 text-sm text-text-2"
    >
      <span className={`size-2 rounded-full ${ok ? 'bg-status-matched' : 'bg-status-extra'}`} />
      {unknown ? t('health.unknown') : ok ? t('health.ok') : t('health.degraded')}
    </span>
  );
}
