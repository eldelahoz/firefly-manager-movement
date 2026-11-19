import { useTranslation } from 'react-i18next';
import { CircleCheck } from 'lucide-react';
import { formatMoney } from '@/shared/lib/money';
import { Skeleton } from '@/shared/ui/Skeleton';
import { TABLE_WRAPPER } from './tableLayout';

export type SummaryCounts = { matched: number; missing: number; extra: number; ignored: number };

const SEGMENTS: { key: keyof SummaryCounts; dot: string }[] = [
  { key: 'matched', dot: 'bg-status-matched' },
  { key: 'missing', dot: 'bg-status-missing' },
  { key: 'extra', dot: 'bg-status-extra' },
  { key: 'ignored', dot: 'bg-border-strong' },
];

/** What is left to do: missing count and amounts, a proportional bar, and per-group counts. */
export function ResultsSummary({
  counts,
  outCents,
  inCents,
  failed,
  currency,
}: {
  counts: SummaryCounts;
  outCents: number;
  inCents: number;
  failed: number;
  currency: string;
}) {
  const { t, i18n } = useTranslation();
  const money = (c: number) => formatMoney(Math.abs(c), currency, i18n.language);
  const flows = [
    outCents ? t('results.summary.out', { amount: money(outCents) }) : null,
    inCents ? t('results.summary.in', { amount: money(inCents) }) : null,
  ].filter(Boolean);
  return (
    <section
      aria-label={t('results.summary.label')}
      className="grid gap-3 rounded-lg border border-border-subtle bg-surface px-4 py-4 shadow-sm"
    >
      <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
        {counts.missing > 0 ? (
          <>
            <span className="text-xl font-semibold tabular-nums tracking-tight">
              {t('results.summary.toCreate', { count: counts.missing })}
            </span>
            {flows.length > 0 && (
              <span className="text-sm text-text-2 tabular-nums">
                {t('results.summary.notInFf3', { amounts: flows.join(' · ') })}
              </span>
            )}
          </>
        ) : (
          <span className="inline-flex items-center gap-2 text-xl font-semibold tracking-tight text-status-matched-txt">
            <CircleCheck className="size-5" aria-hidden="true" />
            {t('results.summary.allReconciled')}
          </span>
        )}
      </div>
      <div
        aria-hidden="true"
        className="flex h-2 gap-0.5 overflow-hidden rounded-full bg-border-subtle"
      >
        {SEGMENTS.map((s) => (
          <i
            key={s.key}
            className={`ffm-seg block min-w-0 ${s.dot}`}
            style={{ flexGrow: counts[s.key] }}
          />
        ))}
      </div>
      <ul className="flex flex-wrap gap-x-4 gap-y-1 text-sm text-text-2">
        {SEGMENTS.map((s) => (
          <li key={s.key} data-group={s.key} className="inline-flex items-center gap-1.5">
            <span className={`size-2 rounded-full ${s.dot}`} aria-hidden="true" />
            <span className="tabular-nums">
              {t(`results.summary.${s.key}`, { count: counts[s.key] })}
            </span>
          </li>
        ))}
        {failed > 0 && (
          <li data-group="failed" className="font-medium text-danger tabular-nums">
            {t('results.summary.failed', { count: failed })}
          </li>
        )}
      </ul>
    </section>
  );
}

/** Loading state shaped like the summary card and the first table rows. */
export function ResultsSkeleton() {
  const { t } = useTranslation();
  return (
    <div
      role="status"
      aria-label={t('results.summary.loading')}
      aria-busy="true"
      className="space-y-4"
    >
      <div className="grid gap-3 rounded-lg border border-border-subtle bg-surface p-4">
        <Skeleton className="h-6 w-56" />
        <Skeleton className="h-2 w-full" />
        <Skeleton className="h-4 w-80 max-w-full" />
      </div>
      <Skeleton className="h-9 w-72 max-w-full" />
      <div className={TABLE_WRAPPER}>
        {Array.from({ length: 6 }).map((_, i) => (
          <div
            key={i}
            className="flex items-center gap-4 border-b border-border-subtle px-4 py-4 last:border-0"
          >
            <Skeleton className="h-4 w-20" />
            <Skeleton className="h-4 flex-1" />
            <Skeleton className="h-4 w-24" />
          </div>
        ))}
      </div>
    </div>
  );
}
