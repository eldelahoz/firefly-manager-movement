import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { getImports } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';
import { formatDate } from '@/shared/lib/dates';
import { StatusPill } from '@/shared/ui/StatusPill';
import { TableSkeleton } from '@/shared/ui/Skeleton';

export function RecentBatches() {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const { data, isPending } = useQuery({ queryKey: queryKeys.imports, queryFn: getImports });
  if (isPending) return <TableSkeleton rows={4} cols={3} />;
  const top = (data ?? []).slice(0, 8);
  return (
    <section className="rounded-xl border border-border-subtle bg-surface p-4 shadow-sm">
      <h2 className="mb-3 font-semibold">{t('dashboard.recentBatches')}</h2>
      <ul className="divide-y divide-border-subtle">
        {top.map((b) => (
          <li key={b.id}>
            <button
              onClick={() => navigate(b.status !== 'parsed' ? `/results/${b.id}` : '/imports')}
              className="flex w-full items-center justify-between gap-2 py-2.5 text-left hover:text-accent"
            >
              <span className="truncate">{b.filename}</span>
              <span className="flex shrink-0 items-center gap-2 text-xs text-text-3">
                {formatDate(b.created_at.slice(0, 10), i18n.language)}
                <StatusPill status={b.status} />
              </span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
