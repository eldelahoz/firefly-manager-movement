import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Trash2 } from 'lucide-react';
import type { Ff3CachedTxDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { Button } from '@/shared/ui/button';
import { MoneyText } from '@/shared/ui/MoneyText';
import { StatusPill } from '@/shared/ui/StatusPill';
import { DeleteDuplicateDialog } from '@/features/firefly/DeleteDuplicateDialog';
import { GRID_COLS } from './tableLayout';

export function ExtraRow({ tx, currency }: { tx: Ff3CachedTxDTO; currency: string }) {
  const { t, i18n } = useTranslation();
  const [confirming, setConfirming] = useState(false);
  return (
    <div className="border-b border-border-subtle py-3 text-sm">
      <div className={`grid ${GRID_COLS.extra} items-center gap-2`}>
        <span>{formatDate(tx.date, i18n.language)}</span>
        <span className="min-w-0 truncate" title={tx.description}>
          {tx.description}
        </span>
        <MoneyText cents={tx.amount_cents} currency={currency} />
        <StatusPill status={tx.is_duplicate ? 'duplicate' : 'extra'} />
        <span className="flex items-center justify-center">
          {tx.is_duplicate && (
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label={t('dashboard.deleteDuplicate')}
              title={t('dashboard.deleteDuplicate')}
              className="text-status-extra-txt hover:bg-status-extra-bg hover:text-status-extra-txt"
              onClick={() => setConfirming(true)}
            >
              <Trash2 className="size-4" />
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
