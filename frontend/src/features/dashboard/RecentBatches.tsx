import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import type { BatchSummary } from '@/shared/api/dto';
import { useImportWizard } from '@/app/importWizardContext';
import { formatDate } from '@/shared/lib/dates';
import { Button } from '@/shared/ui/button';
import { StatusPill } from '@/shared/ui/StatusPill';
import { Table, Td, Tr } from '@/shared/ui/Table';

export function RecentBatches({ batches }: { batches: BatchSummary[] }) {
  const { t, i18n } = useTranslation();
  const openWizard = useImportWizard();
  return (
    <section aria-label={t('dashboard.recentBatches')} className="min-w-0">
      <h2 className="mb-3 font-semibold">{t('dashboard.recentBatches')}</h2>
      <Table
        headClassNames={['', 'max-sm:hidden']}
        head={[
          t('imports.filename'),
          t('dashboard.date'),
          t('imports.status'),
          <span className="sr-only">{t('dashboard.actions')}</span>,
        ]}
      >
        {batches.slice(0, 8).map((b) => (
          <Tr key={b.id}>
            <Td className="max-w-[8rem] sm:max-w-[24rem]">
              <p className="truncate font-medium">{b.filename}</p>
              <p className="text-xs text-text-2 sm:hidden">
                {formatDate(b.created_at.slice(0, 10), i18n.language)}
              </p>
            </Td>
            <Td className="whitespace-nowrap text-text-2 max-sm:hidden">
              {formatDate(b.created_at.slice(0, 10), i18n.language)}
            </Td>
            <Td className="max-sm:px-2">
              <StatusPill status={b.status} />
            </Td>
            <Td className="text-right max-sm:pl-0">
              {b.status !== 'parsed' ? (
                <Button asChild size="sm" variant="ghost">
                  <Link
                    to={`/results/${b.id}`}
                    aria-label={t('dashboard.reviewBatch', { filename: b.filename })}
                  >
                    {t('dashboard.review')}
                  </Link>
                </Button>
              ) : (
                b.movements_count > 0 && (
                  <Button
                    size="sm"
                    variant="ghost"
                    aria-label={t('dashboard.resumeBatch', { filename: b.filename })}
                    onClick={() => openWizard(b.id)}
                  >
                    {t('imports.resume')}
                  </Button>
                )
              )}
            </Td>
          </Tr>
        ))}
      </Table>
    </section>
  );
}
