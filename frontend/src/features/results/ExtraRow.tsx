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
    <div className="flex items-center justify-between gap-3 border-b border-border-subtle py-3 text-sm">
      <div className="min-w-0">
        <p className="truncate">{tx.description}</p>
        <p className="text-xs text-text-3">
          FF3 {tx.ff3_id} · {formatDate(tx.date, i18n.language)}
        </p>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        <MoneyText cents={tx.amount_cents} currency={currency} />
        {tx.is_duplicate && (
          <>
            <StatusPill status="duplicate" />
            <Button
              variant="ghost"
              size="sm"
              className="text-status-extra-txt hover:bg-status-extra-bg hover:text-status-extra-txt"
              onClick={() => setConfirming(true)}
            >
              {t('dashboard.deleteDuplicate')}
            </Button>
          </>
        )}
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
