import { useReducer } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { X } from 'lucide-react';
import { getAccounts, uploadCsv } from '@/shared/api/endpoints';
import type { UploadResponse } from '@/shared/api/dto';
import { queryKeys } from '@/shared/lib/keys';
import { Button } from '@/shared/ui/button';
import { Dialog, DialogClose, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui/dialog';
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
    <Dialog
      open
      onOpenChange={(o) => {
        if (!o) onClose();
      }}
    >
      <DialogContent showCloseButton={false} className="sm:max-w-2xl">
        <DialogHeader className="flex-row items-center justify-between">
          <DialogTitle className="text-lg">{t('wizard.title')}</DialogTitle>
          <DialogClose asChild>
            <Button variant="ghost" size="icon" aria-label={t('common.close')}>
              <X className="size-5" />
            </Button>
          </DialogClose>
        </DialogHeader>
        <div className="flex gap-1" aria-hidden="true">
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
      </DialogContent>
    </Dialog>
  );
}
