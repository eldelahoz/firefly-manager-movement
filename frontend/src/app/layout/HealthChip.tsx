import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { CircleCheck, CircleHelp, CircleX } from 'lucide-react';
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
  const label = unknown ? t('health.unknown') : ok ? t('health.ok') : t('health.degraded');
  return (
    <span
      // The status is in the title too: with the sidebar collapsed only the icon is visible.
      title={`${label}. ${t('health.probeHint')}`}
      className="flex items-center gap-2 overflow-hidden whitespace-nowrap rounded-full border border-border-subtle bg-canvas px-3 py-1 text-sm text-text-2"
    >
      {/* Shape as well as colour (WCAG 1.4.1): unknown and degraded must be told apart in the collapsed rail. */}
      {unknown ? (
        <CircleHelp aria-hidden="true" className="size-4 shrink-0 text-text-3" />
      ) : ok ? (
        <CircleCheck aria-hidden="true" className="size-4 shrink-0 text-status-matched" />
      ) : (
        <CircleX aria-hidden="true" className="size-4 shrink-0 text-danger" />
      )}
      <span className="ffm-sidebar-label">{label}</span>
    </span>
  );
}
