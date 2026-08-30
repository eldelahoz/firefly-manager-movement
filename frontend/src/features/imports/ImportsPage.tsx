import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useRerun } from '@/app/reconcileContext';
import { getAccounts, getImports } from '@/shared/api/endpoints';
import { formatDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { EmptyState } from '@/shared/ui/EmptyState';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { Eye, Repeat, RefreshCw } from 'lucide-react';
import { usePageHeader } from '@/shared/ui/PageHeader';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { Pill, StatusPill } from '@/shared/ui/StatusPill';
import { Table, Td, Tr } from '@/shared/ui/Table';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { ImportWizard } from './wizard/ImportWizard';

const STATUSES = ['parsed', 'reconciled', 'applied'] as const;

export function ImportsPage() {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const openRerun = useRerun();
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<(typeof STATUSES)[number] | 'all'>('all');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [wizardOpen, setWizardOpen] = useState(false);
  const { data, isPending, isFetching, refetch } = useQuery({
    queryKey: queryKeys.imports,
    queryFn: getImports,
  });
  const { data: accounts } = useQuery({ queryKey: queryKeys.accounts, queryFn: getAccounts });
  const accountNameById = useMemo(() => {
    const map = new Map<number, string>();
    (accounts ?? []).forEach((a) => map.set(a.id, a.name));
    return map;
  }, [accounts]);
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
        .filter((b) => b.filename.toLowerCase().includes(search.toLowerCase()))
        .filter((b) => {
          const day = b.created_at.slice(0, 10);
          return (!dateFrom || day >= dateFrom) && (!dateTo || day <= dateTo);
        }),
    [data, statusFilter, search, dateFrom, dateTo],
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
        <Select
          value={statusFilter}
          onValueChange={(v) => setStatusFilter(v as (typeof STATUSES)[number] | 'all')}
        >
          <SelectTrigger aria-label={t('imports.filterStatus')} className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent position="popper">
            {(['all', ...STATUSES] as const).map((s) => (
              <SelectItem key={s} value={s}>
                {s === 'all' ? t('imports.all') : t(`imports.${s}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Input
          type="date"
          aria-label={t('imports.from')}
          value={dateFrom}
          max={dateTo || undefined}
          onChange={(e) => setDateFrom(e.target.value)}
          className="h-9 w-40"
        />
        <Input
          type="date"
          aria-label={t('imports.to')}
          value={dateTo}
          min={dateFrom || undefined}
          onChange={(e) => setDateTo(e.target.value)}
          className="h-9 w-40"
        />
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
            t('imports.matchedAccount'),
            t('imports.movements'),
            t('imports.status'),
            <span key="actions" className="block text-center">
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
              <Td>
                <Pill>{b.format}</Pill>
              </Td>
              <Td>{b.currency}</Td>
              <Td className="text-text-2">
                {b.account_id != null
                  ? `${b.account_name ?? accountNameById.get(b.account_id) ?? String(b.account_id)} (${b.account_id})`
                  : '—'}
              </Td>
              <Td className="text-text-2">{b.movements_count}</Td>
              <Td>
                <StatusPill status={b.status} />
              </Td>
              <Td className="text-center">
                <div className="flex items-center justify-center gap-1">
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
                </div>
              </Td>
            </Tr>
          ))}
        </Table>
      )}
      <ImportWizard open={wizardOpen} onClose={() => setWizardOpen(false)} />
    </div>
  );
}
