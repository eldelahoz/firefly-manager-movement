import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useRerun } from '@/app/reconcileContext';
import { getImports } from '@/shared/api/endpoints';
import { formatDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { EmptyState } from '@/shared/ui/EmptyState';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { Eye, Info, Repeat, RefreshCw } from 'lucide-react';
import { usePageHeader } from '@/shared/ui/PageHeader';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { StatusPill } from '@/shared/ui/StatusPill';
import { Table, Td, Tr } from '@/shared/ui/Table';
import { BatchDrawer } from './BatchDrawer';
import { ImportWizard } from './wizard/ImportWizard';

const STATUSES = ['parsed', 'reconciled', 'applied'] as const;

export function ImportsPage() {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const openRerun = useRerun();
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<(typeof STATUSES)[number] | 'all'>('all');
  const [drawerId, setDrawerId] = useState<number | null>(null);
  const [wizardOpen, setWizardOpen] = useState(false);
  const { data, isPending, isFetching, refetch } = useQuery({
    queryKey: queryKeys.imports,
    queryFn: getImports,
  });
  const headerActions = useMemo(
    () => (
      <>
        <Button
          variant="outline"
          size="sm"
          disabled={isFetching}
          onClick={() => refetch({ cancelRefetch: true })}
        >
          <RefreshCw className={isFetching ? 'size-4 animate-spin' : 'size-4'} />
        </Button>
        <Button onClick={() => setWizardOpen(true)}>{t('common.newImport')}</Button>
      </>
    ),
    [refetch, isFetching, setWizardOpen, t],
  );
  usePageHeader({ title: t('imports.title'), actions: headerActions });
  const rows = useMemo(
    () =>
      (data ?? [])
        .filter((b) => statusFilter === 'all' || b.status === statusFilter)
        .filter((b) => b.filename.toLowerCase().includes(search.toLowerCase())),
    [data, statusFilter, search],
  );
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <Input
          placeholder={t('imports.search')}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="h-9 flex-1"
        />
        <div className="flex gap-1">
          {(['all', ...STATUSES] as const).map((s) => (
            <Button
              key={s}
              size="sm"
              variant={statusFilter === s ? 'default' : 'outline'}
              className="rounded-full"
              onClick={() => setStatusFilter(s)}
            >
              {s === 'all' ? t('imports.all') : t(`imports.${s}`)}
            </Button>
          ))}
        </div>
      </div>
      {isPending ? (
        <TableSkeleton />
      ) : rows.length === 0 ? (
        <EmptyState title={t('imports.empty')} />
      ) : (
        <Table
          head={[
            t('imports.filename'),
            t('imports.createdAt'),
            t('imports.format'),
            t('imports.currency'),
            t('imports.status'),
            <span key="actions" className="block text-right">
              {t('imports.actions')}
            </span>,
          ]}
        >
          {rows.map((b) => (
            <Tr key={b.id}>
              <Td className="font-medium">{b.filename}</Td>
              <Td className="text-text-2">
                {formatDate(b.created_at.slice(0, 10), i18n.language)}
              </Td>
              <Td>{b.format}</Td>
              <Td>{b.currency}</Td>
              <Td>
                <StatusPill status={b.status} />
              </Td>
              <Td className="text-right">
                <div className="flex items-center justify-end gap-1">
                  {b.status !== 'parsed' && (
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      aria-label={t('imports.viewResults')}
                      title={t('imports.viewResults')}
                      onClick={() => navigate(`/results/${b.id}`)}
                    >
                      <Eye className="size-4" />
                    </Button>
                  )}
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    aria-label={t('imports.rerun')}
                    title={t('imports.rerun')}
                    onClick={() => openRerun(b.id)}
                  >
                    <Repeat className="size-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    aria-label={t('imports.detail')}
                    title={t('imports.detail')}
                    onClick={() => setDrawerId(b.id)}
                  >
                    <Info className="size-4" />
                  </Button>
                </div>
              </Td>
            </Tr>
          ))}
        </Table>
      )}
      <BatchDrawer batchId={drawerId} onClose={() => setDrawerId(null)} />
      <ImportWizard open={wizardOpen} onClose={() => setWizardOpen(false)} />
    </div>
  );
}
