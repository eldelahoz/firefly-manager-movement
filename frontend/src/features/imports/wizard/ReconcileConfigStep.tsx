import { useEffect, useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getAccounts, getImport, postReconcile } from '@/shared/api/endpoints';
import { addDays } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { TableSkeleton } from '@/shared/ui/Skeleton';

const TOLERANCES = [0, 1, 2, 3];

export function ReconcileConfigStep({
  batchId,
  currency,
  submitting,
  error,
  onStart,
  onError,
  onDone,
}: {
  batchId: number | null;
  currency: string;
  submitting: boolean;
  error: string | null;
  onStart: () => void;
  onError: (message: string) => void;
  onDone: (batchId: number) => void;
}) {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [accountId, setAccountId] = useState('');
  const [tolerance, setTolerance] = useState(0);
  const [range, setRange] = useState<{ start: string; end: string } | null>(null);
  const accounts = useQuery({ queryKey: queryKeys.accounts, queryFn: getAccounts, staleTime: 300_000 });
  const detail = useQuery({
    queryKey: queryKeys.import(batchId ?? -1),
    queryFn: () => getImport(batchId!),
    enabled: batchId !== null,
  });
  useEffect(() => {
    if (!range && detail.data && detail.data.movements.length > 0) {
      const dates = detail.data.movements.map((m) => m.date).sort();
      setRange({ start: addDays(dates[0], -7), end: addDays(dates[dates.length - 1], 7) });
    }
  }, [detail.data, range]);
  const currencyMismatch = useMemo(
    () => !!accounts.data?.find((a) => a.id === accountId && a.currency !== currency),
    [accounts.data, accountId, currency],
  );
  const reconcile = useMutation({
    mutationFn: () =>
      postReconcile(batchId!, {
        ff3_account_id: accountId,
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
      <label className="block text-sm text-text-2">
        {t('wizard.account')}
        <select
          value={accountId}
          onChange={(e) => setAccountId(e.target.value)}
          className="mt-1 h-11 w-full rounded-lg border border-border-subtle bg-surface px-2 text-text-1"
        >
          <option value="">—</option>
          {accounts.data?.map((a) => (
            <option key={a.id} value={a.id}>
              {a.name} ({a.currency})
            </option>
          ))}
        </select>
      </label>
      <div>
        <p className="mb-1 text-sm text-text-2">{t('wizard.tolerance')}</p>
        <div className="flex gap-1">
          {TOLERANCES.map((d) => (
            <button
              key={d}
              onClick={() => setTolerance(d)}
              className={`h-11 flex-1 rounded-lg ${tolerance === d ? 'bg-accent text-white' : 'border border-border-subtle bg-surface text-text-2'}`}
            >
              {d === 0 ? '0' : `±${d}`}
            </button>
          ))}
        </div>
      </div>
      {range && (
        <div className="flex gap-2">
          <label className="flex-1 text-sm text-text-2">
            {t('dashboard.browseFrom')}
            <input
              type="date"
              value={range.start}
              onChange={(e) => setRange({ ...range, start: e.target.value })}
              className="mt-1 h-11 w-full rounded-lg border border-border-subtle bg-surface px-2"
            />
          </label>
          <label className="flex-1 text-sm text-text-2">
            {t('dashboard.browseTo')}
            <input
              type="date"
              value={range.end}
              onChange={(e) => setRange({ ...range, end: e.target.value })}
              className="mt-1 h-11 w-full rounded-lg border border-border-subtle bg-surface px-2"
            />
          </label>
        </div>
      )}
      {currencyMismatch && (
        <p role="alert" className="rounded-lg bg-status-missing-bg px-3 py-2 text-sm text-status-missing-txt">
          {t('wizard.currencyMismatch')}
        </p>
      )}
      {error && !currencyMismatch && (
        <p role="alert" className="rounded-lg bg-status-missing-bg px-3 py-2 text-sm text-status-missing-txt">
          {error}
        </p>
      )}
      <button
        disabled={!ready}
        onClick={() => {
          onStart();
          reconcile.mutate();
        }}
        className="h-11 w-full rounded-lg bg-accent font-medium text-white disabled:opacity-50"
      >
        {submitting ? t('wizard.reconciling') : t('common.finish')}
      </button>
    </div>
  );
}
