import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getImports, getResults } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';

export function KpiCards() {
  const { t } = useTranslation();
  const imports = useQuery({ queryKey: queryKeys.imports, queryFn: getImports });
  const lastReconciled = imports.data?.find((b) => b.status !== 'parsed');
  const results = useQuery({
    queryKey: queryKeys.results(lastReconciled?.id ?? -1),
    queryFn: () => getResults(lastReconciled!.id),
    enabled: !!lastReconciled,
  });
  if (imports.isPending) return null;
  const cards = [
    { label: t('dashboard.totalBatches'), value: String(imports.data?.length ?? 0) },
    { label: t('status.matched'), value: results.data ? String(results.data.matched.length) : '—' },
    { label: t('status.missing'), value: results.data ? String(results.data.missing.length) : '—' },
    { label: t('status.extra'), value: results.data ? String(results.data.extra.length) : '—' },
  ];
  return (
    <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
      {cards.map((c) => (
        <div
          key={c.label}
          className="rounded-xl border border-border-subtle bg-surface p-4 shadow-sm"
        >
          <p className="text-sm text-text-2">{c.label}</p>
          <p className="mt-1 text-2xl font-bold" aria-label={c.label}>
            {c.value}
          </p>
        </div>
      ))}
    </div>
  );
}
