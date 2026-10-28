import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useState } from 'react';
import { X } from 'lucide-react';
import { Button } from '@/shared/ui/button';
import { Dialog, DialogClose, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui/dialog';
import { ReconcileConfigStep } from './ReconcileConfigStep';

export function ReconcileDialog({
  batchId,
  onClose,
}: {
  batchId: number | null;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  if (batchId === null) return null;
  // The dialog stays mounted in ReconcileProvider, so its state outlives each run.
  const close = () => {
    setSubmitting(false);
    setError(null);
    onClose();
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => {
        if (!o) close();
      }}
    >
      <DialogContent showCloseButton={false} className="sm:max-w-md">
        <DialogHeader className="flex-row items-center justify-between">
          <DialogTitle>{t('imports.reconcileTitle')}</DialogTitle>
          <DialogClose asChild>
            <Button variant="ghost" size="icon" aria-label={t('common.close')}>
              <X className="size-5" />
            </Button>
          </DialogClose>
        </DialogHeader>
        <ReconcileConfigStep
          batchId={batchId}
          submitting={submitting}
          error={error}
          onStart={() => setSubmitting(true)}
          onError={(m) => {
            setSubmitting(false);
            setError(m);
          }}
          onDone={(id) => {
            close();
            navigate(`/results/${id}`);
          }}
          onCancel={close}
        />
      </DialogContent>
    </Dialog>
  );
}
