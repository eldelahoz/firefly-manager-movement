import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useState } from 'react';
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
  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={t('imports.rerun')}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
    >
      <div className="w-full max-w-md rounded-xl bg-surface p-6 shadow-lg">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="font-semibold">{t('imports.rerun')}</h2>
          <button
            onClick={onClose}
            aria-label={t('common.close')}
            className="h-11 px-2 text-text-2"
          >
            ✕
          </button>
        </div>
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
            onClose();
            navigate(`/results/${id}`);
          }}
        />
      </div>
    </div>
  );
}
