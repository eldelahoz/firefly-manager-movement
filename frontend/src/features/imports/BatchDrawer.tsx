import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getImport } from '@/shared/api/endpoints';
import { formatDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { MoneyText } from '@/shared/ui/MoneyText';
import { StatusPill } from '@/shared/ui/StatusPill';
import { Table, Td, Tr } from '@/shared/ui/Table';
import { TableSkeleton } from '@/shared/ui/Skeleton';

export function BatchDrawer({ batchId, onClose }: { batchId: number | null; onClose: () => void }) {
  const { t, i18n } = useTranslation();
  const { data, isPending } = useQuery({
    queryKey: queryKeys.import(batchId ?? -1),
    queryFn: () => getImport(batchId!),
    enabled: batchId !== null,
  });
  if (batchId === null) return null;
  return (
    <div
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 bg-black/50"
      onClick={onClose}
    >
      <div
        className="absolute right-0 top-0 h-full w-full max-w-md overflow-y-auto bg-surface p-6 shadow-lg"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-4 flex items-center justify-between">
          <h2 className="truncate font-semibold">{data?.batch.filename ?? ''}</h2>
          <button
            onClick={onClose}
            aria-label={t('common.close')}
            className="h-11 px-2 text-text-2"
          >
            ✕
          </button>
        </div>
        {isPending || !data ? (
          <TableSkeleton rows={3} cols={3} />
        ) : data.movements.length === 0 ? (
          <p className="text-sm text-text-3">{t('imports.probeHint')}</p>
        ) : (
          <Table
            head={['#', t('imports.createdAt'), t('imports.description'), t('imports.status')]}
          >
            {data.movements.map((m) => (
              <Tr key={m.id}>
                <Td>{m.line_no}</Td>
                <Td>{formatDate(m.date, i18n.language)}</Td>
                <Td>
                  <span className="block max-w-40 truncate">{m.description}</span>
                  <MoneyText cents={m.amount_cents} currency={data.batch.currency} />
                </Td>
                <Td>
                  <StatusPill status={m.status} />
                </Td>
              </Tr>
            ))}
          </Table>
        )}
      </div>
    </div>
  );
}
