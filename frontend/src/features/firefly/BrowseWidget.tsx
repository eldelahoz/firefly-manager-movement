import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { Ff3TxDTO } from '@/shared/api/dto';
import { getAccounts, postAccountTransactions } from '@/shared/api/endpoints';
import { addDays, formatDate, toIsoDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { MoneyText } from '@/shared/ui/MoneyText';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { DeleteDuplicateDialog } from './DeleteDuplicateDialog';

export function BrowseWidget() {
  const { t, i18n } = useTranslation();
  const now = new Date();
  const [accountId, setAccountId] = useState('');
  const [start, setStart] = useState(
    toIsoDate(new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1))),
  );
  const [end, setEnd] = useState(toIsoDate(now));
  const [pendingDelete, setPendingDelete] = useState<Ff3TxDTO | null>(null);
  const accounts = useQuery({
    queryKey: queryKeys.accounts,
    queryFn: getAccounts,
    staleTime: 300_000,
  });
  const txs = useQuery({
    queryKey: queryKeys.accountTxs(accountId, start, end),
    queryFn: () => postAccountTransactions(accountId, start, end),
    enabled: !!accountId,
  });
  const currency = useMemo(
    () => accounts.data?.find((a) => a.id === accountId)?.currency ?? 'EUR',
    [accounts.data, accountId],
  );
  return (
    <section className="rounded-xl border border-border-subtle bg-surface p-4 shadow-sm">
      <h2 className="mb-3 font-semibold">{t('dashboard.browseTitle')}</h2>
      <div className="flex flex-wrap gap-2">
        <select
          aria-label={t('wizard.account')}
          value={accountId}
          onChange={(e) => setAccountId(e.target.value)}
          className="h-11 min-w-40 rounded-lg border border-border-subtle bg-surface px-2"
        >
          <option value="">{t('dashboard.browseSelectAccount')}</option>
          {accounts.data?.map((a) => (
            <option key={a.id} value={a.id}>
              {a.name}
            </option>
          ))}
        </select>
        <label className="flex items-center gap-1 text-sm text-text-2">
          {t('dashboard.browseFrom')}
          <input
            type="date"
            value={start}
            max={addDays(end, -1)}
            onChange={(e) => setStart(e.target.value)}
            className="h-11 rounded-lg border border-border-subtle bg-surface px-2"
          />
        </label>
        <label className="flex items-center gap-1 text-sm text-text-2">
          {t('dashboard.browseTo')}
          <input
            type="date"
            value={end}
            min={addDays(start, 1)}
            onChange={(e) => setEnd(e.target.value)}
            className="h-11 rounded-lg border border-border-subtle bg-surface px-2"
          />
        </label>
      </div>
      <div className="mt-4">
        {!accountId ? (
          <p className="py-8 text-center text-text-3">{t('dashboard.browseSelectAccount')}</p>
        ) : txs.isPending ? (
          <TableSkeleton rows={4} cols={3} />
        ) : txs.isError ? (
          <p role="alert" className="py-4 text-center text-status-extra-txt">
            {t('health.degraded')}
          </p>
        ) : txs.data.length === 0 ? (
          <p className="py-8 text-center text-text-3">{t('dashboard.browseEmpty')}</p>
        ) : (
          <ul className="divide-y divide-border-subtle">
            {txs.data.map((tx) => (
              <li key={tx.id} className="flex items-center justify-between gap-2 py-2.5">
                <span className="min-w-0">
                  <span className="block truncate text-sm">{tx.description}</span>
                  <span className="text-xs text-text-3">{formatDate(tx.date, i18n.language)}</span>
                </span>
                <span className="flex shrink-0 items-center gap-2">
                  <MoneyText cents={tx.amount_cents} currency={currency} />
                  <button
                    onClick={() => setPendingDelete(tx)}
                    className="rounded-lg px-2 py-1 text-xs text-status-extra-txt hover:bg-status-extra-bg"
                  >
                    {t('dashboard.deleteDuplicate')}
                  </button>
                </span>
              </li>
            ))}
          </ul>
        )}
      </div>
      <DeleteDuplicateDialog
        tx={pendingDelete}
        currency={currency}
        onClose={() => setPendingDelete(null)}
      />
    </section>
  );
}
