import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import type { BatchSummary } from '@/shared/api/dto';
import { getResults } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';
import { useImportWizard } from '@/app/importWizardContext';
import { Button } from '@/shared/ui/button';
import { Skeleton } from '@/shared/ui/Skeleton';
import { SummaryBreakdown } from '@/features/results/ResultsSummary';

const ROW = 'flex flex-wrap items-center justify-between gap-x-4 gap-y-2 py-3';

/** What needs the user: the latest reconciled batch's missing rows and unfinished imports (RF-18).
 *  Only the latest reconciled batch's results are fetched (reviewer #10: no per-batch fetches). */
export function NeedsYou({ batches }: { batches: BatchSummary[] }) {
  const { t } = useTranslation();
  const openWizard = useImportWizard();
  const latest = batches.find((b) => b.status !== 'parsed');
  const unfinished = batches.filter((b) => b.status === 'parsed' && b.movements_count > 0);
  const results = useQuery({
    queryKey: queryKeys.results(latest?.id ?? -1),
    queryFn: () => getResults(latest!.id),
    enabled: !!latest,
  });
  const counts = results.data && {
    matched: results.data.matched.length,
    missing: results.data.missing.length,
    extra: results.data.extra.length,
    ignored: results.data.ignored.length,
  };
  return (
    <section
      aria-label={t('dashboard.needsYou')}
      className="rounded-xl border border-border-subtle bg-surface px-4 py-2 shadow-sm"
    >
      <h2 className="pt-2 font-semibold">{t('dashboard.needsYou')}</h2>
      <ul className="divide-y divide-border-subtle">
        {latest && (
          <li className="grid gap-3 py-3">
            <div className={`${ROW} py-0`}>
              <p className="min-w-0 text-sm">
                <span className="font-medium break-all">{latest.filename}</span>
                {counts && (
                  <span className="text-text-2">
                    {' · '}
                    {counts.missing > 0
                      ? t('results.summary.toCreate', { count: counts.missing })
                      : t('results.summary.allReconciled')}
                  </span>
                )}
              </p>
              <Button asChild size="sm" variant={counts?.missing ? 'default' : 'outline'}>
                <Link
                  to={`/results/${latest.id}`}
                  aria-label={t('dashboard.reviewBatch', { filename: latest.filename })}
                >
                  {t('dashboard.review')}
                </Link>
              </Button>
            </div>
            {counts ? (
              <SummaryBreakdown counts={counts} />
            ) : (
              results.isPending && <Skeleton className="h-2 w-full rounded-full" />
            )}
          </li>
        )}
        {unfinished.map((b) => (
          <li key={b.id} className={ROW}>
            <p className="min-w-0 text-sm">
              <span className="font-medium break-all">{b.filename}</span>
              <span className="text-text-2">
                {' · '}
                {t('dashboard.notFinished')}
              </span>
            </p>
            <Button
              size="sm"
              variant="outline"
              aria-label={t('dashboard.resumeBatch', { filename: b.filename })}
              onClick={() => openWizard(b.id)}
            >
              {t('imports.resume')}
            </Button>
          </li>
        ))}
        {!latest && unfinished.length === 0 && (
          <li className="py-3 text-sm text-text-2">{t('dashboard.nothingNeeded')}</li>
        )}
      </ul>
    </section>
  );
}
