import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getAccounts, getImport, postReconcile } from '@/shared/api/endpoints';
import { addDays } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { TableSkeleton } from '@/shared/ui/Skeleton';

const TOLERANCES = [0, 1, 2, 3];

export function ReconcileConfigStep({
  batchId,
  initialAccountId,
  submitting,
  error,
  onStart,
  onError,
  onDone,
}: {
  batchId: number | null;
  initialAccountId?: string;
  submitting: boolean;
  error: string | null;
  onStart: () => void;
  onError: (message: string) => void;
  onDone: (batchId: number) => void;
}) {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [accountId, setAccountId] = useState(initialAccountId ?? '');
  const [tolerance, setTolerance] = useState(0);
  const [rangeOverride, setRangeOverride] = useState<{ start: string; end: string } | null>(null);
  const accounts = useQuery({
    queryKey: queryKeys.accounts,
    queryFn: getAccounts,
    staleTime: 300_000,
  });
  const detail = useQuery({
    queryKey: queryKeys.import(batchId ?? -1),
    queryFn: () => getImport(batchId!),
    enabled: batchId !== null,
  });
  const computedRange = useMemo(() => {
    if (!detail.data || detail.data.movements.length === 0) return null;
    const dates = detail.data.movements.map((m) => m.date).sort();
    return { start: addDays(dates[0], -7), end: addDays(dates[dates.length - 1], 7) };
  }, [detail.data]);
  const currency = detail.data?.batch.currency ?? '';
  const range = rangeOverride ?? computedRange;
  const selectedAccount = accounts.data?.find((a) => a.id === Number(accountId));
  const currencyMismatch = useMemo(
    () => !!currency && !!selectedAccount && selectedAccount.currency !== currency,
    [accounts.data, accountId, currency, selectedAccount],
  );
  const reconcile = useMutation({
    mutationFn: () =>
      postReconcile(batchId!, {
        ff3_account_id: Number(accountId),
        window_start: range!.start,
        window_end: range!.end,
        tolerance_days: tolerance,
      }),
    onSuccess: (res) => {
      void qc.invalidateQueries({ queryKey: queryKeys.results(res.batch_id) });
      void qc.invalidateQueries({ queryKey: queryKeys.imports });
      onDone(res.batch_id);
    },
    onError: (e: { message: string }) => onError(e.message),
  });
  if (detail.isPending && batchId !== null) return <TableSkeleton rows={2} cols={2} />;
  const ready = !!accountId && !!range && !currencyMismatch && !submitting;
  return (
    <div className="space-y-4">
      <div>
        <p className="mb-1 text-sm text-text-2">{t('wizard.account')}</p>
        <Select
          value={accountId === '' ? 'none' : accountId}
          onValueChange={(v) => setAccountId(v === 'none' ? '' : v)}
        >
          <SelectTrigger className="w-full" aria-label={t('wizard.account')}>
            <SelectValue placeholder="—" />
          </SelectTrigger>
          <SelectContent position="popper">
            <SelectItem value="none">—</SelectItem>
            {accounts.data?.map((a) => (
              <SelectItem key={a.id} value={String(a.id)}>
                {a.name} ({a.currency})
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div>
        <p className="mb-1 text-sm text-text-2">{t('wizard.tolerance')}</p>
        <div className="flex gap-1">
          {TOLERANCES.map((d) => (
            <Button
              key={d}
              variant={tolerance === d ? 'default' : 'outline'}
              className="flex-1"
              onClick={() => setTolerance(d)}
            >
              {d === 0 ? '0' : `±${d}`}
            </Button>
          ))}
        </div>
      </div>
      {range && (
        <div className="flex gap-2">
          <label className="flex-1 text-sm text-text-2">
            {t('dashboard.browseFrom')}
            <Input
              type="date"
              value={range.start}
              onChange={(e) => setRangeOverride({ ...range, start: e.target.value })}
              className="mt-1"
            />
          </label>
          <label className="flex-1 text-sm text-text-2">
            {t('dashboard.browseTo')}
            <Input
              type="date"
              value={range.end}
              onChange={(e) => setRangeOverride({ ...range, end: e.target.value })}
              className="mt-1"
            />
          </label>
        </div>
      )}
      {currencyMismatch && (
        <p
          role="alert"
          className="rounded-lg bg-status-missing-bg px-3 py-2 text-sm text-status-missing-txt"
        >
          {t('wizard.currencyMismatch', {
            batchCurrency: currency,
            accountCurrency: selectedAccount!.currency,
          })}
        </p>
      )}
      {error && !currencyMismatch && (
        <p
          role="alert"
          className="rounded-lg bg-status-missing-bg px-3 py-2 text-sm text-status-missing-txt"
        >
          {error}
        </p>
      )}
      <Button
        disabled={!ready}
        onClick={() => {
          onStart();
          reconcile.mutate();
        }}
        className="w-full"
      >
        {submitting ? t('wizard.reconciling') : t('common.finish')}
      </Button>
    </div>
  );
}
