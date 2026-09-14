import { useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getAccounts, getImport, postReconcile } from '@/shared/api/endpoints';
import { addDays } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { WizardFooter } from './WizardShell';

const TOLERANCES = [0, 1, 2, 3];

export function ReconcileConfigStep({
  batchId,
  initialAccountId,
  submitting,
  error,
  onStart,
  onError,
  onDone,
  onBack,
  onCancel,
}: {
  batchId: number | null;
  initialAccountId?: string;
  submitting: boolean;
  error: string | null;
  onStart: () => void;
  onError: (message: string) => void;
  onDone: (batchId: number) => void;
  onBack?: () => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const detail = useQuery({
    queryKey: queryKeys.import(batchId ?? -1),
    queryFn: () => getImport(batchId!),
    enabled: batchId !== null,
  });
  // Locked account (RF-04 v0.8): decided at import, never editable here.
  // `||` (not `??`) so an empty-string wizard state falls back to the persisted batch account.
  const accountId = initialAccountId || String(detail.data?.batch.account_id ?? '');
  const [tolerance, setTolerance] = useState(0);
  const [rangeOverride, setRangeOverride] = useState<{ start: string; end: string } | null>(null);
  const accounts = useQuery({
    queryKey: queryKeys.accounts,
    queryFn: getAccounts,
    staleTime: 300_000,
  });
  const computedRange = useMemo(() => {
    if (!detail.data || detail.data.movements.length === 0) return null;
    const dates = detail.data.movements.map((m) => m.date).sort();
    const start = addDays(dates[0], -7);
    const end = addDays(dates[dates.length - 1], 7);
    if (!start || !end) return null;
    return { start, end };
  }, [detail.data]);
  const currency = detail.data?.batch.currency ?? '';
  const range = rangeOverride ?? computedRange;
  const selectedAccount = accounts.data?.find((a) => a.id === Number(accountId));
  const currencyMismatch = useMemo(
    () => !!currency && !!selectedAccount && selectedAccount.currency !== currency,
    [currency, selectedAccount],
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
  const helper = currencyMismatch
    ? t('wizard.helperMismatch')
    : error
      ? t('wizard.helperError')
      : null;
  return (
    <div className="flex h-full flex-col">
      <div className="space-y-5">
        <div>
          <p className="mb-1 text-[13px] text-text-2">{t('wizard.account')}</p>
          {selectedAccount && (
            <p className="flex items-center gap-2 text-sm text-text-1">
              {selectedAccount.name}
              <span className="rounded-full bg-muted px-2.5 py-0.5 text-[11px] font-medium text-text-2">
                {t('wizard.accountLockedHint')}
              </span>
            </p>
          )}
        </div>
        <div>
          <p className="mb-1 text-[13px] text-text-2">{t('wizard.tolerance')}</p>
          <div className="flex max-w-[480px]">
            {TOLERANCES.map((d, i) => (
              <Button
                key={d}
                variant={tolerance === d ? 'default' : 'outline'}
                onClick={() => setTolerance(d)}
                className={`h-10 flex-1 ${i === 0 ? 'rounded-l-lg' : ''} ${i === TOLERANCES.length - 1 ? 'rounded-r-lg' : ''} ${i > 0 ? '-ml-px rounded-none' : ''}`}
              >
                {d === 0 ? '0' : `±${d}`}
              </Button>
            ))}
          </div>
        </div>
        {range && (
          <div className="flex max-w-[480px] gap-2">
            <label className="flex-1 text-[13px] text-text-2">
              {t('dashboard.browseFrom')}
              <Input
                type="date"
                value={range.start}
                onChange={(e) => setRangeOverride({ ...range, start: e.target.value })}
                className="mt-1"
              />
            </label>
            <label className="flex-1 text-[13px] text-text-2">
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
        {currency && (
          <div>
            <p className="mb-1 text-[13px] text-text-2">{t('wizard.currency')}</p>
            <div className="flex items-center gap-2">
              <p className="text-sm text-text-1">{t('wizard.currencyFromBatch', { currency })}</p>
              {selectedAccount && (
                <span
                  className={`rounded-full px-2.5 py-0.5 text-[11px] font-medium ${
                    currencyMismatch
                      ? 'bg-status-extra-bg text-status-extra-txt'
                      : 'bg-status-matched-bg text-status-matched-txt'
                  }`}
                >
                  {currencyMismatch
                    ? t('wizard.currencyMismatchPill')
                    : t('wizard.currencyMatches')}
                </span>
              )}
            </div>
          </div>
        )}
        {currencyMismatch && (
          <div
            role="alert"
            className="flex max-w-[640px] items-start gap-2 rounded-lg bg-status-missing-bg p-3"
          >
            <span className="text-lg leading-none font-semibold text-status-missing-txt">⚠</span>
            <span className="text-sm font-medium text-status-missing-txt">
              {t('wizard.currencyMismatch', {
                batchCurrency: currency,
                accountCurrency: selectedAccount!.currency,
              })}
            </span>
          </div>
        )}
        {error && !currencyMismatch && (
          <div
            role="alert"
            className="flex max-w-[640px] items-start gap-2 rounded-lg bg-status-missing-bg p-3"
          >
            <span className="text-lg leading-none font-semibold text-status-missing-txt">⚠</span>
            <span className="text-sm font-medium text-status-missing-txt">{error}</span>
          </div>
        )}
      </div>
      <WizardFooter
        onCancel={onCancel}
        cancelLabel={t('common.cancel')}
        onBack={onBack}
        backLabel={t('common.back')}
        primaryLabel={submitting ? t('wizard.reconciling') : t('wizard.reconcile')}
        onPrimary={() => {
          onStart();
          reconcile.mutate();
        }}
        primaryDisabled={!ready}
        primaryHelper={helper}
      />
    </div>
  );
}
