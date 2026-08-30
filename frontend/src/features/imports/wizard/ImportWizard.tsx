import { useReducer } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { getAccounts, uploadCsv } from '@/shared/api/endpoints';
import type { UploadResponse } from '@/shared/api/dto';
import { queryKeys } from '@/shared/lib/keys';
import { Dialog } from '@/shared/ui/dialog';
import { WizardShell } from './WizardShell';
import { initialWizardState, wizardReducer } from './state';
import { UploadStep } from './UploadStep';
import { PreviewStep } from './PreviewStep';
import { MappingStep } from './MappingStep';
import { ReconcileConfigStep } from './ReconcileConfigStep';

const STEP_ORDER = ['upload', 'preview', 'mapping', 'reconcile'] as const;

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
  const headerStep = state.step === 'submitting' ? 'reconcile' : state.step;
  const idx = Math.max(0, STEP_ORDER.indexOf(headerStep as (typeof STEP_ORDER)[number]));
  const stepNames = STEP_ORDER.map((s) => t(`wizard.steps.${s}`));
  const subtitle =
    state.step === 'upload'
      ? state.error
        ? t('wizard.couldNotParse')
        : upload.isPending
          ? t('wizard.subtitleParsing')
          : t('wizard.subtitleUpload')
      : state.step === 'mapping'
        ? t('wizard.subtitleMapping')
        : state.step === 'preview'
          ? state.probe && state.probe.invalid_count > 0
            ? t('wizard.subtitlePreviewInvalid')
            : t('wizard.subtitlePreview')
          : t('wizard.subtitleReconcile');
  return (
    <Dialog
      open
      onOpenChange={(o) => {
        if (!o) onClose();
      }}
    >
      <WizardShell
        title={t('wizard.stepTitle', { n: idx + 1, step: stepNames[idx] })}
        subtitle={subtitle}
        steps={stepNames}
        current={idx}
      >
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
            onCancel={onClose}
            error={state.error}
          />
        )}
        {state.step === 'preview' && !state.probe && (
          <div
            className="flex flex-1 flex-col items-center justify-center gap-3"
            aria-live="polite"
          >
            <p className="text-sm font-semibold">{state.file?.name}</p>
            <div className="h-2 w-full max-w-md overflow-hidden rounded bg-border-subtle">
              <div className="h-full w-1/3 animate-pulse rounded bg-accent" />
            </div>
            <p className="text-xs text-text-2">
              {t('wizard.parsingFile', { filename: state.file?.name ?? '' })}
            </p>
          </div>
        )}
        {state.step === 'preview' && state.probe && (
          <PreviewStep
            probe={state.probe}
            currency={state.currency}
            onNext={() => dispatch({ type: 'NEXT_FROM_PREVIEW' })}
            onBack={() => dispatch({ type: 'RESET' })}
            onClose={onClose}
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
            onBack={() => dispatch({ type: 'BACK_FROM_MAPPING' })}
            onCancel={onClose}
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
            onBack={() => dispatch({ type: 'BACK_FROM_RECONCILE' })}
            onCancel={onClose}
          />
        )}
      </WizardShell>
    </Dialog>
  );
}
