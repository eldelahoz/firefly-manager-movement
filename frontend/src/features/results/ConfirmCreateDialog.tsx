import { useTranslation } from 'react-i18next';
import type { Ff3Account } from '@/shared/api/dto';
import type { Customization } from './customization';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/shared/ui/dialog';

export function ConfirmCreateDialog({
  open,
  batchId,
  filename,
  movements,
  accounts,
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
  onConfirm: () => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  function accountName(id?: number) {
    if (!id) return null;
    return accounts.find((a) => a.id === id)?.name ?? null;
  }
  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o) onClose();
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
            const cpName = accountName(c.counterpartyId);
            return (
              <div key={m.id} className="border-b border-border-subtle pb-2 text-sm">
                <p className="truncate font-medium">{m.description}</p>
                <MoneyText cents={m.amountCents} currency="EUR" />
                {cpName && <p className="text-text-2">→ {cpName}</p>}
                {c.category && <p className="text-text-3">Category: {c.category}</p>}
                {c.budget && <p className="text-text-3">Budget: {c.budget}</p>}
                {c.tags && c.tags.length > 0 && (
                  <p className="text-text-3">Tags: {c.tags.join(', ')}</p>
                )}
                {c.notes && <p className="text-text-3">Notes: {c.notes}</p>}
                {c.attachment && <p className="text-text-3">Attachment: {c.attachment.filename}</p>}
              </div>
            );
          })}
        </div>
        <p className="text-xs text-text-3">
          {t('results.originTagNote', { tag: `ffm:${filename}:${batchId}` })}
        </p>
        <DialogFooter>
          <Button variant="cancel" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button onClick={onConfirm}>{t('results.confirm')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
