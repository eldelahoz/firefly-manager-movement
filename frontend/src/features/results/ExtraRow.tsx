import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Ff3CachedTxDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { Button } from '@/shared/ui/button';
import { MoneyText } from '@/shared/ui/MoneyText';
import { StatusPill } from '@/shared/ui/StatusPill';
import { DeleteDuplicateDialog } from '@/features/firefly/DeleteDuplicateDialog';

export function ExtraRow({ tx, currency }: { tx: Ff3CachedTxDTO; currency: string }) {
  const { t, i18n } = useTranslation();
  const [confirming, setConfirming] = useState(false);
  return (
    <div className="border-b border-border-subtle py-3 text-sm">
      <div className="grid grid-cols-[7rem_1fr_8rem_7rem_9rem] items-center gap-2">
        <span>{formatDate(tx.date, i18n.language)}</span>
        <span className="min-w-0 truncate" title={tx.description}>
          {tx.description}
        </span>
        <MoneyText cents={tx.amount_cents} currency={currency} />
        <StatusPill status={tx.is_duplicate ? 'duplicate' : 'extra'} />
        <span>
          {tx.is_duplicate && (
            <Button
              variant="ghost"
              size="sm"
              className="text-status-extra-txt hover:bg-status-extra-bg hover:text-status-extra-txt"
              onClick={() => setConfirming(true)}
            >
              {t('dashboard.deleteDuplicate')}
            </Button>
          )}
        </span>
      </div>
      <DeleteDuplicateDialog
        tx={
          confirming
            ? {
                id: tx.ff3_id,
                date: tx.date,
                amount_cents: tx.amount_cents,
                description: tx.description,
              }
            : null
        }
        currency={currency}
        onClose={() => setConfirming(false)}
      />
    </div>
  );
}
