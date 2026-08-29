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
import { PageHeader } from '@/shared/ui/PageHeader';
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
  const { data, isPending } = useQuery({ queryKey: queryKeys.imports, queryFn: getImports });
  const rows = useMemo(
    () =>
      (data ?? [])
        .filter((b) => statusFilter === 'all' || b.status === statusFilter)
        .filter((b) => b.filename.toLowerCase().includes(search.toLowerCase())),
    [data, statusFilter, search],
  );
  return (
    <div className="space-y-4">
      <PageHeader
        title={t('imports.title')}
        actions={<Button onClick={() => setWizardOpen(true)}>{t('common.newImport')}</Button>}
      />
      <div className="flex flex-wrap items-center gap-2">
        <Input
          placeholder={t('imports.search')}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="h-9 flex-1"
        />
        <div className="flex gap-1">
          {(['all', ...STATUSES] as const).map((s) => (
            <button
              key={s}
              onClick={() => setStatusFilter(s)}
              className={`rounded-full px-3 py-1.5 text-sm ${statusFilter === s ? 'bg-accent text-white' : 'border border-border-subtle bg-surface text-text-2'}`}
            >
              {s === 'all' ? t('imports.all') : t(`imports.${s}`)}
            </button>
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
            t('imports.actions'),
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
              <Td>
                <div className="flex gap-2">
                  {b.status !== 'parsed' && (
                    <button
                      onClick={() => navigate(`/results/${b.id}`)}
                      className="rounded-lg px-2 py-1 text-sm text-accent hover:bg-accent-soft"
                    >
                      {t('imports.viewResults')}
                    </button>
                  )}
                  <button
                    onClick={() => openRerun(b.id)}
                    className="rounded-lg px-2 py-1 text-sm text-text-2 hover:bg-canvas"
                  >
                    {t('imports.rerun')}
                  </button>
                  <button
                    onClick={() => setDrawerId(b.id)}
                    className="rounded-lg px-2 py-1 text-sm text-text-2 hover:bg-canvas"
                  >
                    {t('imports.detail')}
                  </button>
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
