import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import type { Ff3TxDTO } from '@/shared/api/dto';
import { ApiError } from '@/shared/api/client';
import { deleteTransaction } from '@/shared/api/endpoints';
import { formatDate } from '@/shared/lib/dates';
import { Button } from '@/shared/ui/button';
import { Dialog, DialogContent, DialogTitle } from '@/shared/ui/dialog';
import { MoneyText } from '@/shared/ui/MoneyText';

export function DeleteDuplicateDialog({
  tx,
  currency,
  onClose,
}: {
  tx: Ff3TxDTO | null;
  currency: string;
  onClose: () => void;
}) {
  const { t, i18n } = useTranslation();
  const [confirmed, setConfirmed] = useState(false);
  const qc = useQueryClient();
  const del = useMutation({
    mutationFn: (id: number) => deleteTransaction(id),
    onSuccess: () => {
      toast.success(t('deleteDialog.deleted'));
      void qc.invalidateQueries({ queryKey: ['account-txs'] });
      void qc.invalidateQueries({ queryKey: ['results'] });
      setConfirmed(false);
      onClose();
    },
    onError: (e: ApiError) => {
      if (e.status === 404) toast.error(t('deleteDialog.notDeletable'));
      else toast.error(t('common.error'));
    },
  });
  if (!tx) return null;
  return (
    <Dialog
      open
      onOpenChange={(o) => {
        if (!o) onClose();
      }}
    >
      <DialogContent showCloseButton={false} className="sm:max-w-md">
        <DialogTitle className="text-base text-destructive">{t('deleteDialog.title')}</DialogTitle>
        {!confirmed ? (
          <>
            <p className="mt-2 text-sm text-text-2">{t('deleteDialog.recap')}</p>
            <dl className="mt-4 space-y-2 text-sm">
              <div className="flex justify-between gap-4">
                <dt className="text-text-3">FF3</dt>
                <dd>{tx.id}</dd>
              </div>
              <div className="flex justify-between gap-4">
                <dt className="text-text-3">{t('imports.createdAt')}</dt>
                <dd>{formatDate(tx.date, i18n.language)}</dd>
              </div>
              <div className="flex justify-between gap-4">
                <dt className="text-text-3">{t('wizard.amountColumn')}</dt>
                <dd>
                  <MoneyText cents={tx.amount_cents} currency={currency} />
                </dd>
              </div>
              <div className="flex justify-between gap-4">
                <dt className="text-text-3">{t('wizard.descriptionColumn')}</dt>
                <dd className="truncate">{tx.description}</dd>
              </div>
            </dl>
            <p className="mt-3 text-xs text-text-3">{t('deleteDialog.auditNote')}</p>
            <div className="mt-6 flex justify-end gap-2">
              <Button variant="cancel" size="lg" onClick={onClose}>
                {t('common.cancel')}
              </Button>
              <Button variant="destructive" size="lg" onClick={() => setConfirmed(true)}>
                {t('common.delete')}
              </Button>
            </div>
          </>
        ) : (
          <>
            <p className="mt-4 font-medium">{t('deleteDialog.confirmTitle')}</p>
            <div className="mt-6 flex justify-end gap-2">
              <Button variant="cancel" size="lg" onClick={() => setConfirmed(false)}>
                {t('common.back')}
              </Button>
              <Button
                variant="destructive"
                size="lg"
                onClick={() => del.mutate(tx.id)}
                disabled={del.isPending}
              >
                {t('common.confirm')}
              </Button>
            </div>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
