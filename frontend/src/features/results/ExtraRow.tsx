import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Trash2 } from 'lucide-react';
import type { Ff3CachedTxDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { Button } from '@/shared/ui/button';
import { MoneyText } from '@/shared/ui/MoneyText';
import { StatusPill } from '@/shared/ui/StatusPill';
import { DeleteDuplicateDialog } from '@/features/firefly/DeleteDuplicateDialog';
import { GRID_COLS, GROUP_ACCENT } from './tableLayout';

export function ExtraRow({
  tx,
  currency,
  isLast = false,
}: {
  tx: Ff3CachedTxDTO;
  currency: string;
  isLast?: boolean;
}) {
  const { t, i18n } = useTranslation();
  const [confirming, setConfirming] = useState(false);
  return (
    <>
      <div
        role="row"
        className={`grid ${GRID_COLS.extra} items-center gap-3 px-4 py-3 text-sm transition-colors ${
          isLast ? '' : 'border-b border-border-subtle'
        } ${GROUP_ACCENT.extra.rowHover}`}
      >
        <span role="cell">{formatDate(tx.date, i18n.language)}</span>
        <span role="cell" className="min-w-0 truncate" title={tx.description}>
          {tx.description}
        </span>
        <div role="cell" className="text-right">
          <MoneyText cents={tx.amount_cents} currency={currency} />
        </div>
        <div role="cell" className="flex justify-center">
          <StatusPill status={tx.is_duplicate ? 'duplicate' : 'extra'} />
        </div>
        <span role="cell" className="flex items-center justify-end">
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
    </>
  );
}
