import { useTranslation } from 'react-i18next';
import { LoaderCircle } from 'lucide-react';
import type { Ff3Account } from '@/shared/api/dto';
import type { Customization } from './customization';
import { transactionType } from './customization';
import { TransactionTypeIcon } from './TransactionTypeIcon';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/shared/ui/dialog';

export function ConfirmCreateDialog({
  open,
  batchId,
  filename,
  movements,
  accounts,
  currency,
  pending,
  onConfirm,
  onClose,
}: {
  open: boolean;
  batchId: number;
  filename: string;
  movements: {
    id: number;
    description: string;
    amountCents: number;
    customization: Customization;
  }[];
  accounts: Ff3Account[];
  currency: string;
  pending: boolean;
  onConfirm: () => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  function counterpartyText(c: Customization, out: boolean) {
    const cp = c.counterparty;
    if (!cp) return t('results.cp.cash');
    if ('name' in cp)
      return t(out ? 'results.cp.newExpense' : 'results.cp.newRevenue', { name: cp.name });
    return accounts.find((a) => a.id === cp.id)?.name;
  }
  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o && !pending) onClose();
      }}
    >
      <DialogContent className="max-h-[80vh] overflow-y-auto sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('results.confirmTitle')}</DialogTitle>
          <p className="text-sm text-text-2">
            {t('results.confirmCount', { count: movements.length })}
          </p>
        </DialogHeader>
        <div className="space-y-3">
          {movements.map((m) => {
            const c = m.customization;
            const out = m.amountCents < 0;
            const type = transactionType(out ? 'withdrawal' : 'deposit', c.counterparty, accounts);
            return (
              <div key={m.id} className="border-b border-border-subtle pb-2 text-sm">
                <p className="truncate font-medium">{m.description}</p>
                <MoneyText cents={m.amountCents} currency={currency} />
                {/* An account missing from the loaded lists: neither its name nor its type is known. */}
                <p className="flex items-center gap-1 text-text-2">
                  {type && <TransactionTypeIcon type={type} />}
                  <span className="min-w-0 break-words">
                    {type ? `${t(`results.cp.type.${type}`)} · ${counterpartyText(c, out)}` : '—'}
                  </span>
                </p>
                {c.category && (
                  <p className="text-text-3">
                    {t('results.confirmField.category', { value: c.category })}
                  </p>
                )}
                {c.budget && (
                  <p className="text-text-3">
                    {t('results.confirmField.budget', { value: c.budget })}
                  </p>
                )}
                {c.tags && c.tags.length > 0 && (
                  <p className="text-text-3">
                    {t('results.confirmField.tags', { value: c.tags.join(', ') })}
                  </p>
                )}
                {c.notes && (
                  <p className="text-text-3">
                    {t('results.confirmField.notes', { value: c.notes })}
                  </p>
                )}
                {c.attachment && (
                  <p className="text-text-3">
                    {t('results.confirmField.attachment', { value: c.attachment.filename })}
                  </p>
                )}
              </div>
            );
          })}
        </div>
        <p className="text-xs text-text-3">
          {t('results.originNoteNote', { note: `ffm:${filename}:${batchId}` })}
        </p>
        <DialogFooter>
          <Button variant="cancel" disabled={pending} onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button disabled={pending} onClick={onConfirm} aria-busy={pending}>
            {pending && <LoaderCircle className="animate-spin" aria-hidden="true" />}
            {pending ? t('results.creating') : t('results.confirm')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
