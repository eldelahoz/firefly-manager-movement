import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { X } from 'lucide-react';
import { getImport } from '@/shared/api/endpoints';
import { formatDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { Button } from '@/shared/ui/button';
import { MoneyText } from '@/shared/ui/MoneyText';
import { StatusPill } from '@/shared/ui/StatusPill';
import { Table, Td, Tr } from '@/shared/ui/Table';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { Sheet, SheetClose, SheetContent, SheetHeader, SheetTitle } from '@/shared/ui/sheet';

export function BatchDrawer({ batchId, onClose }: { batchId: number | null; onClose: () => void }) {
  const { t, i18n } = useTranslation();
  const { data, isPending } = useQuery({
    queryKey: queryKeys.import(batchId ?? -1),
    queryFn: () => getImport(batchId!),
    enabled: batchId !== null,
  });
  if (batchId === null) return null;
  return (
    <Sheet
      open
      onOpenChange={(o) => {
        if (!o) onClose();
      }}
    >
      <SheetContent
        side="right"
        showCloseButton={false}
        className="w-full gap-0 overflow-y-auto p-6 sm:max-w-md"
      >
        <SheetHeader className="flex-row items-center justify-between p-0">
          <SheetTitle className="truncate">{data?.batch.filename ?? ''}</SheetTitle>
          <SheetClose asChild>
            <Button variant="ghost" size="icon" aria-label={t('common.close')}>
              <X className="size-5" />
            </Button>
          </SheetClose>
        </SheetHeader>
        <div className="mt-4">
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
      </SheetContent>
    </Sheet>
  );
}
