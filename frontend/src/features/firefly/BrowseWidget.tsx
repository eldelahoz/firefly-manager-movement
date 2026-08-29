import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { Ff3TxDTO } from '@/shared/api/dto';
import { getAccounts, postAccountTransactions } from '@/shared/api/endpoints';
import { addDays, formatDate, toIsoDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { StatusPill } from '@/shared/ui/StatusPill';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
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
    () => accounts.data?.find((a) => a.id === Number(accountId))?.currency ?? 'EUR',
    [accounts.data, accountId],
  );
  return (
    <section className="rounded-xl border border-border-subtle bg-surface p-4 shadow-sm">
      <h2 className="mb-3 font-semibold">{t('dashboard.browseTitle')}</h2>
      <div className="flex flex-wrap gap-2">
        <Select
          value={accountId === '' ? 'none' : accountId}
          onValueChange={(v) => setAccountId(v === 'none' ? '' : v)}
        >
          <SelectTrigger className="min-w-50" aria-label={t('wizard.account')}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent position="popper">
            <SelectItem value="none">{t('dashboard.browseSelectAccount')}</SelectItem>
            {accounts.data?.map((a) => (
              <SelectItem key={a.id} value={String(a.id)}>
                {a.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <label className="flex items-center gap-1 text-sm text-text-2">
          {t('dashboard.browseFrom')}
          <Input
            type="date"
            value={start}
            max={addDays(end, -1)}
            onChange={(e) => setStart(e.target.value)}
            className="w-auto"
          />
        </label>
        <label className="flex items-center gap-1 text-sm text-text-2">
          {t('dashboard.browseTo')}
          <Input
            type="date"
            value={end}
            min={addDays(start, 1)}
            onChange={(e) => setEnd(e.target.value)}
            className="w-auto"
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
              <li key={tx.id} className="flex items-center gap-3 py-2.5">
                <span className="w-20 shrink-0 text-xs text-text-3">
                  {formatDate(tx.date, i18n.language)}
                </span>
                <span className="min-w-0 flex-1 truncate text-sm">{tx.description}</span>
                <MoneyText cents={tx.amount_cents} currency={currency} signed colored />
                {tx.status && <StatusPill status={tx.status} />}
                {tx.status === 'duplicate' && (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-status-extra-txt hover:bg-status-extra-bg hover:text-status-extra-txt"
                    onClick={() => setPendingDelete(tx)}
                  >
                    {t('dashboard.deleteDuplicate')}
                  </Button>
                )}
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
