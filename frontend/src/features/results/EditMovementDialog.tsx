import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Ff3Account, MovementDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import type { Customization } from './customization';
import { PersonalizeEditor } from './PersonalizeEditor';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui/dialog';

export function EditMovementDialog({
  movement,
  initial,
  defaultTag,
  currency,
  accounts,
  tags,
  categories,
  budgets,
  onSave,
  onClose,
}: {
  movement: MovementDTO;
  initial: Customization;
  defaultTag: string | null;
  currency: string;
  accounts: Ff3Account[];
  tags: string[];
  categories: string[];
  budgets: string[];
  onSave: (c: Customization) => void;
  onClose: () => void;
}) {
  const { t, i18n } = useTranslation();
  const [dirty, setDirty] = useState(false);
  const [confirmDiscard, setConfirmDiscard] = useState(false);
  const direction: 'withdrawal' | 'deposit' = movement.amount_cents < 0 ? 'withdrawal' : 'deposit';

  function requestClose() {
    if (dirty) setConfirmDiscard(true);
    else onClose();
  }

  return (
    <Dialog
      open
      onOpenChange={(o) => {
        if (!o) requestClose();
      }}
    >
      <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{t('results.editTitle')}</DialogTitle>
          <p className="flex items-center gap-2 text-sm text-text-2">
            <span>{formatDate(movement.date, i18n.language)}</span>
            <span className="min-w-0 truncate" title={movement.description}>
              {movement.description}
            </span>
            <MoneyText cents={movement.amount_cents} currency={currency} />
          </p>
        </DialogHeader>
        <PersonalizeEditor
          initial={initial}
          defaultTag={defaultTag}
          direction={direction}
          accounts={accounts}
          tags={tags}
          categories={categories}
          budgets={budgets}
          onDirtyChange={setDirty}
          onSave={onSave}
          onCancel={requestClose}
          submitLabel={t('results.save')}
        />
      </DialogContent>
      {confirmDiscard && (
        <Dialog
          open
          onOpenChange={(o) => {
            if (!o) setConfirmDiscard(false);
          }}
        >
          <DialogContent className="sm:max-w-sm">
            <DialogHeader>
              <DialogTitle>{t('results.discardTitle')}</DialogTitle>
              <p className="text-sm text-text-2">{t('results.discardBody')}</p>
            </DialogHeader>
            <div className="flex justify-end gap-2">
              <Button variant="cancel" onClick={() => setConfirmDiscard(false)}>
                {t('results.keepEditing')}
              </Button>
              <Button onClick={onClose}>{t('results.discard')}</Button>
            </div>
          </DialogContent>
        </Dialog>
      )}
    </Dialog>
  );
}
