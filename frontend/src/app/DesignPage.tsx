import { useTranslation } from 'react-i18next';
import { StatusPill, type PillStatus } from '@/shared/ui/StatusPill';
import { MoneyText } from '@/shared/ui/MoneyText';
import { EmptyState } from '@/shared/ui/EmptyState';
import { Skeleton, TableSkeleton } from '@/shared/ui/Skeleton';
import { Table, Td, Tr } from '@/shared/ui/Table';
import { Button } from '@/shared/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/shared/ui/dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui/tabs';

const STATUSES: PillStatus[] = [
  'imported',
  'matched',
  'missing',
  'created',
  'ignored',
  'duplicate',
  'extra',
  'already_created',
  'failed',
];

export function DesignPage() {
  const { t } = useTranslation();
  return (
    <div className="space-y-8">
      <section>
        <h2 className="mb-3 text-lg font-semibold">StatusPill</h2>
        <div className="flex flex-wrap gap-2">
          {STATUSES.map((s) => (
            <StatusPill key={s} status={s} />
          ))}
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">MoneyText</h2>
        <div className="flex flex-wrap gap-4">
          <MoneyText cents={123456} currency="EUR" />
          <MoneyText cents={-990} currency="EUR" />
          <MoneyText cents={0} currency="EUR" />
          <MoneyText cents={500} currency="EUR" signed />
          <MoneyText cents={-500} currency="EUR" signed />
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">Table</h2>
        <Table head={['File', 'Date', 'Status', 'Amount']}>
          <Tr>
            <Td>bank-export.csv</Td>
            <Td>{t('common.loading')}</Td>
            <Td>
              <StatusPill status="matched" />
            </Td>
            <Td>
              <MoneyText cents={123456} currency="EUR" />
            </Td>
          </Tr>
          <Tr>
            <Td>wallet.csv</Td>
            <Td>{t('common.loading')}</Td>
            <Td>
              <StatusPill status="missing" />
            </Td>
            <Td>
              <MoneyText cents={-2500} currency="EUR" />
            </Td>
          </Tr>
        </Table>
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">Skeleton</h2>
        <div className="mb-4 flex items-center gap-2">
          <Skeleton className="h-6 w-40" />
          <Skeleton className="size-10 rounded-full" />
        </div>
        <TableSkeleton rows={3} cols={4} />
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">EmptyState</h2>
        <EmptyState title="No imports yet" action={<Button>{t('common.newImport')}</Button>} />
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">Dialog</h2>
        <Dialog>
          <DialogTrigger asChild>
            <Button variant="destructive">{t('deleteDialog.title')}</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle className="text-destructive">{t('deleteDialog.title')}</DialogTitle>
              <DialogDescription>{t('deleteDialog.recap')}</DialogDescription>
            </DialogHeader>
            <p className="text-xs text-text-3">{t('deleteDialog.auditNote')}</p>
            <DialogFooter>
              <Button variant="cancel">{t('common.cancel')}</Button>
              <Button variant="destructive">{t('common.confirm')}</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">Tabs</h2>
        <Tabs defaultValue="matched">
          <TabsList>
            <TabsTrigger value="matched">{t('results.matched')} (3)</TabsTrigger>
            <TabsTrigger value="missing">{t('results.missing')} (1)</TabsTrigger>
            <TabsTrigger value="extra">{t('results.extra')} (2)</TabsTrigger>
          </TabsList>
          <TabsContent value="matched" className="pt-4">
            Matched content
          </TabsContent>
          <TabsContent value="missing" className="pt-4">
            Missing content
          </TabsContent>
          <TabsContent value="extra" className="pt-4">
            Extra content
          </TabsContent>
        </Tabs>
      </section>
    </div>
  );
}
