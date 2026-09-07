import { useMemo, useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useRerun } from '@/app/reconcileContext';
import { useImportWizard } from '@/app/importWizardContext';
import { getAccounts, getImports, deleteImport } from '@/shared/api/endpoints';
import type { BatchSummary } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { EmptyState } from '@/shared/ui/EmptyState';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { Eye, Play, Repeat, RefreshCw, Trash2 } from 'lucide-react';
import { usePageHeader } from '@/shared/ui/PageHeader';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { Pill, StatusPill } from '@/shared/ui/StatusPill';
import { Table, Td, Tr } from '@/shared/ui/Table';
import { Dialog, DialogClose, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';

const STATUSES = ['parsed', 'reconciled', 'applied'] as const;

export function ImportsPage() {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const openRerun = useRerun();
  const openWizard = useImportWizard();
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<(typeof STATUSES)[number] | 'all'>('all');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<BatchSummary | null>(null);
  const del = useMutation({
    mutationFn: () => deleteImport(deleteTarget!.id),
    onSuccess: () => {
      setDeleteTarget(null);
      void refetch();
    },
  });
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
        <Button onClick={() => openWizard()}>{t('common.newImport')}</Button>
      </>
    ),
    [refetch, isFetching, openWizard, t],
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
            <Tr
              key={b.id}
              className={b.status === 'parsed' ? 'bg-status-missing-bg/50' : undefined}
              title={
                b.status !== 'parsed'
                  ? undefined
                  : b.movements_count > 0
                    ? t('imports.pendingReconcile')
                    : t('imports.probeHint')
              }
            >
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
                  {b.status === 'parsed' && b.movements_count > 0 && (
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      aria-label={t('imports.resume')}
                      title={t('imports.resume')}
                      onClick={() => openWizard(b.id)}
                    >
                      <Play className="size-4" />
                    </Button>
                  )}
                  {b.status !== 'parsed' && (
                    <>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        aria-label={t('imports.viewResults')}
                        title={t('imports.viewResults')}
                        onClick={() => navigate(`/results/${b.id}`)}
                      >
                        <Eye className="size-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        aria-label={t('imports.rerun')}
                        title={t('imports.rerun')}
                        onClick={() => openRerun(b.id)}
                      >
                        <Repeat className="size-4" />
                      </Button>
                    </>
                  )}
                  {b.status === 'parsed' && (
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      aria-label={t('imports.delete')}
                      title={t('imports.delete')}
                      className="text-destructive"
                      onClick={() => setDeleteTarget(b)}
                    >
                      <Trash2 className="size-4" />
                    </Button>
                  )}
                </div>
              </Td>
            </Tr>
          ))}
        </Table>
      )}
      <Dialog
        open={deleteTarget !== null}
        onOpenChange={(o) => {
          if (!o && !del.isPending) setDeleteTarget(null);
        }}
      >
        <DialogContent showCloseButton={false} className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{t('imports.deleteTitle')}</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-text-2">
            {t('imports.deleteBody', {
              filename: deleteTarget?.filename ?? '',
              count: deleteTarget?.movements_count ?? 0,
            })}
          </p>
          {del.isError && (
            <p role="alert" className="text-sm text-destructive">
              {(del.error as { message: string })?.message ?? t('common.error')}
            </p>
          )}
          <div className="mt-2 flex justify-end gap-2">
            <DialogClose asChild>
              <Button variant="outline" disabled={del.isPending}>
                {t('common.cancel')}
              </Button>
            </DialogClose>
            <Button variant="destructive" disabled={del.isPending} onClick={() => del.mutate()}>
              {t('common.confirm')}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
