import { useReducer } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { getAccounts, uploadCsv } from '@/shared/api/endpoints';
import type { UploadResponse } from '@/shared/api/dto';
import { queryKeys } from '@/shared/lib/keys';
import { initialWizardState, wizardReducer } from './state';
import { UploadStep } from './UploadStep';
import { PreviewStep } from './PreviewStep';
import { MappingStep } from './MappingStep';
import { ReconcileConfigStep } from './ReconcileConfigStep';

const STEP_ORDER = ['upload', 'preview', 'mapping', 'reconcile-config'] as const;

export function ImportWizard({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { t } = useTranslation();
  const [state, dispatch] = useReducer(wizardReducer, initialWizardState);
  const qc = useQueryClient();
  const navigate = useNavigate();
  const accounts = useQuery({
    queryKey: queryKeys.accounts,
    queryFn: getAccounts,
    staleTime: 300_000,
  });
  const upload = useMutation({
    mutationFn: (args: { file: File; currency: string; columns?: Record<string, string> }) =>
      uploadCsv(args.file, { currency: args.currency, ...args.columns }),
    onSuccess: (res) => {
      dispatch({ type: 'UPLOAD_SUCCESS', probe: res as UploadResponse });
      void qc.invalidateQueries({ queryKey: queryKeys.imports });
    },
    onError: (e: { message: string }) => dispatch({ type: 'UPLOAD_ERROR', message: e.message }),
  });
  if (!open) return null;
  const activeIndex = STEP_ORDER.indexOf(state.step as (typeof STEP_ORDER)[number]);
  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={t('wizard.title')}
      className="fixed inset-0 z-50 flex items-end justify-center bg-black/50 md:items-center md:p-4"
    >
      <div className="flex max-h-[95vh] w-full max-w-2xl flex-col overflow-y-auto rounded-t-xl bg-surface p-6 shadow-lg md:rounded-xl">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-semibold">{t('wizard.title')}</h2>
          <button
            onClick={onClose}
            aria-label={t('common.close')}
            className="h-11 px-2 text-text-2"
          >
            ✕
          </button>
        </div>
        <div className="mb-6 flex gap-1" aria-hidden="true">
          {STEP_ORDER.map((s, i) => (
            <span
              key={s}
              className={`h-1.5 flex-1 rounded ${i <= activeIndex ? 'bg-accent' : 'bg-border-subtle'}`}
            />
          ))}
        </div>
        {state.step === 'upload' && (
          <UploadStep
            accounts={accounts.data}
            accountId={state.accountId}
            onAccount={(id, currency) => dispatch({ type: 'SET_ACCOUNT', accountId: id, currency })}
            onFile={(file) => {
              dispatch({ type: 'UPLOAD_START', file });
              upload.mutate({ file, currency: state.currency });
            }}
            uploading={upload.isPending}
          />
        )}
        {state.step === 'preview' && !state.probe && (
          <p aria-live="polite">{t('wizard.uploading')}</p>
        )}
        {state.step === 'preview' && state.probe && (
          <PreviewStep
            probe={state.probe}
            currency={state.currency}
            onNext={() => dispatch({ type: 'NEXT_FROM_PREVIEW' })}
            onBack={() => dispatch({ type: 'RESET' })}
          />
        )}
        {state.step === 'mapping' && state.probe && state.file && (
          <MappingStep
            probe={state.probe}
            onMapped={(columns) =>
              upload.mutate(
                { file: state.file!, currency: state.currency, columns },
                {
                  onSuccess: (res) => {
                    dispatch({ type: 'MAPPING_SUCCESS', probe: res as UploadResponse });
                    void qc.invalidateQueries({ queryKey: queryKeys.imports });
                  },
                },
              )
            }
          />
        )}
        {(state.step === 'reconcile-config' || state.step === 'submitting') && (
          <ReconcileConfigStep
            batchId={state.finalBatchId ?? state.probe?.batch_id ?? null}
            initialAccountId={state.accountId}
            submitting={state.step === 'submitting'}
            error={state.error}
            onStart={() => dispatch({ type: 'RECONCILE_START' })}
            onError={(message) => dispatch({ type: 'RECONCILE_ERROR', message })}
            onDone={(batchId) => {
              dispatch({ type: 'RESET' });
              onClose();
              navigate(`/results/${batchId}`);
            }}
          />
        )}
      </div>
    </div>
  );
}
