import { useCallback, useEffect, useReducer, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import {
  getAccounts,
  getImport,
  getImports,
  uploadCsv,
  deleteImport,
} from '@/shared/api/endpoints';
import type { UploadResponse, Ff3Account } from '@/shared/api/dto';
import { queryKeys } from '@/shared/lib/keys';
import { Dialog } from '@/shared/ui/dialog';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { WizardShell } from './WizardShell';
import { initialWizardState, wizardReducer, type WizardState, type OriginNotice } from './state';
import { suggestAccount } from './originSuggest';
import { UploadStep } from './UploadStep';
import { PreviewStep } from './PreviewStep';
import { MappingStep } from './MappingStep';
import { ReconcileConfigStep } from './ReconcileConfigStep';

function computeNotice(res: UploadResponse, accounts: Ff3Account[] | undefined): OriginNotice {
  const values = res.origin_account_values ?? [];
  if (values.length === 0) return null;
  const s = suggestAccount(values, accounts ?? []);
  if (s.account) return { kind: 'auto', accountName: s.account.name };
  return { kind: 'warn', values: s.values };
}

const STEP_ORDER = ['upload', 'preview', 'mapping', 'reconcile'] as const;

export function ImportWizard({
  open,
  onClose,
  resumeBatchId,
}: {
  open: boolean;
  onClose: () => void;
  resumeBatchId?: number | null;
}) {
  const { t } = useTranslation();
  const [state, dispatch] = useReducer(wizardReducer, resumeBatchId, (batchId) =>
    batchId
      ? ({
          ...initialWizardState,
          step: 'reconcile-config' as const,
          finalBatchId: batchId,
        } as WizardState)
      : initialWizardState,
  );
  const qc = useQueryClient();
  const navigate = useNavigate();
  const removeFile = useCallback(async () => {
    // Discarding a parsed file cleans up its probe batch (no orphans)
    if (state.probe) {
      try {
        await deleteImport(state.probe.batch_id);
      } catch (e) {
        dispatch({ type: 'UPLOAD_ERROR', message: (e as { message: string }).message });
        return;
      }
      void qc.invalidateQueries({ queryKey: queryKeys.imports });
    }
    dispatch({ type: 'REMOVE_FILE' });
  }, [state.probe, qc]);

  const accounts = useQuery({
    queryKey: queryKeys.accounts,
    queryFn: getAccounts,
    staleTime: 300_000,
  });
  const imports = useQuery({
    queryKey: queryKeys.imports,
    queryFn: getImports,
    staleTime: 30_000,
  });
  const resumeDetail = useQuery({
    queryKey: queryKeys.import(resumeBatchId ?? -1),
    queryFn: () => getImport(resumeBatchId!),
    enabled: !!resumeBatchId && state.probe === null,
  });
  useEffect(() => {
    if (!resumeDetail.data || state.probe !== null) return;
    const { batch, movements } = resumeDetail.data;
    const probe: UploadResponse = {
      batch_id: batch.id,
      filename: batch.filename,
      file_hash: batch.file_hash,
      // Persisted at upload: the mapping step lists these columns on resume (RF-02, RF-18)
      headers: resumeDetail.data.headers,
      detected_delimiter: resumeDetail.data.detected_delimiter,
      total_rows: movements.length,
      valid_count: batch.movements_count,
      invalid_count: 0,
      mapping_used: null,
      origin_account_values: [],
      valid: movements.map((m) => ({
        line_no: m.line_no,
        date: m.date,
        amount_cents: m.amount_cents,
        description: m.description,
      })),
      invalid: [],
    };
    dispatch({
      type: 'RESUME_INIT',
      probe,
      accountId: batch.account_id != null ? String(batch.account_id) : '',
      currency: batch.currency,
      batchId: batch.id,
    });
  }, [resumeDetail.data, state.probe]);
  // Sample rows for the mapping-step date-format preview (first 8 KB of the file).
  const [sampleRows, setSampleRows] = useState<string[][]>([]);
  useEffect(() => {
    if (state.step !== 'mapping' || !state.file || !state.probe) return;
    let cancelled = false;
    void state.file
      .slice(0, 8192)
      .text()
      .then((text) => {
        if (cancelled) return;
        const lines = text.split(/\r?\n/).filter((l) => l.trim() !== '');
        const delim = state.probe!.detected_delimiter;
        setSampleRows(lines.slice(1).map((l) => l.split(delim)));
      });
    return () => {
      cancelled = true;
    };
  }, [state.step, state.file, state.probe]);
  const upload = useMutation({
    mutationFn: (args: {
      file: File;
      currency: string;
      account_id?: string;
      columns?: Record<string, string>;
    }) =>
      uploadCsv(args.file, {
        currency: args.currency,
        ...(args.account_id ? { account_id: args.account_id } : {}),
        ...args.columns,
      }),
    onSuccess: (res) => {
      const notice = computeNotice(res as UploadResponse, accounts.data);
      if (notice?.kind === 'auto') {
        const acct = accounts.data?.find((a) => a.name === notice.accountName);
        if (acct)
          dispatch({ type: 'SET_ACCOUNT', accountId: String(acct.id), currency: acct.currency });
      }
      dispatch({ type: 'UPLOAD_SUCCESS', probe: res as UploadResponse, originNotice: notice });
      void qc.invalidateQueries({ queryKey: queryKeys.imports });
    },
    onError: (e: { message: string }) => dispatch({ type: 'UPLOAD_ERROR', message: e.message }),
  });
  if (!open) return null;
  if (resumeBatchId && state.probe === null) {
    if (resumeDetail.isError) {
      return (
        <Dialog
          open
          onOpenChange={(o) => {
            if (!o) onClose();
          }}
        >
          <WizardShell
            title={t('wizard.stepTitle', { n: 4, step: t('wizard.steps.reconcile') })}
            subtitle={t('wizard.subtitleReconcile')}
            steps={STEP_ORDER.map((s) => t(`wizard.steps.${s}`))}
            current={3}
          >
            <div
              className="flex flex-1 flex-col items-center justify-center gap-3"
              aria-live="polite"
            >
              <p className="text-sm font-semibold text-status-missing-txt">
                {resumeDetail.error?.message ?? 'Failed to load import'}
              </p>
            </div>
          </WizardShell>
        </Dialog>
      );
    }
    return (
      <Dialog
        open
        onOpenChange={(o) => {
          if (!o) onClose();
        }}
      >
        <WizardShell
          title={t('wizard.stepTitle', { n: 4, step: t('wizard.steps.reconcile') })}
          subtitle={t('wizard.subtitleReconcile')}
          steps={STEP_ORDER.map((s) => t(`wizard.steps.${s}`))}
          current={3}
        >
          <TableSkeleton rows={2} cols={2} />
        </WizardShell>
      </Dialog>
    );
  }
  const headerStep =
    state.step === 'submitting' || state.step === 'reconcile-config' ? 'reconcile' : state.step;
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
            onFile={async (file) => {
              const dup = (imports.data ?? []).some(
                (b) => b.filename.toLowerCase() === file.name.toLowerCase(),
              );
              if (dup) {
                dispatch({ type: 'UPLOAD_ERROR', message: t('imports.duplicateFilename') });
                return;
              }
              // Replacing a parsed file cleans up its probe batch first (no orphans)
              if (state.probe) {
                try {
                  await deleteImport(state.probe.batch_id);
                } catch (e) {
                  dispatch({ type: 'UPLOAD_ERROR', message: (e as { message: string }).message });
                  return;
                }
                void qc.invalidateQueries({ queryKey: queryKeys.imports });
              }
              // Stage only — the upload/parse runs on Next (RF-01 v0.1)
              dispatch({ type: 'UPLOAD_START', file });
            }}
            onNext={async () => {
              if (!state.file) return;
              // Re-parse always: delete a previous probe batch first (RF-16/RF-17-safe)
              if (state.probe) {
                try {
                  await deleteImport(state.probe.batch_id);
                } catch (e) {
                  dispatch({ type: 'UPLOAD_ERROR', message: (e as { message: string }).message });
                  return;
                }
              }
              upload.mutate({
                file: state.file,
                currency: state.currency,
                account_id: state.accountId,
              });
            }}
            file={state.file}
            onRemoveFile={removeFile}
            uploading={upload.isPending}
            onCancel={onClose}
            error={state.error}
          />
        )}
        {state.step === 'preview' && state.probe && (
          <PreviewStep
            probe={state.probe}
            currency={state.currency}
            originNotice={state.originNotice}
            onNext={() => dispatch({ type: 'NEXT_FROM_PREVIEW' })}
            onBack={() => dispatch({ type: 'BACK_FROM_PREVIEW' })}
            onClose={onClose}
          />
        )}
        {state.step === 'mapping' && state.probe && (
          <MappingStep
            probe={state.probe}
            sampleRows={sampleRows}
            onMapped={async (columns) => {
              const probeId = state.probe?.batch_id;
              if (probeId == null) return;
              if (state.file) {
                // Normal flow: delete probe and re-upload file with mapping
                try {
                  await deleteImport(probeId);
                } catch (e) {
                  dispatch({ type: 'UPLOAD_ERROR', message: (e as { message: string }).message });
                  return;
                }
                upload.mutate(
                  {
                    file: state.file,
                    currency: state.currency,
                    account_id: state.accountId,
                    columns,
                  },
                  {
                    onSuccess: (res) => {
                      const notice = computeNotice(res as UploadResponse, accounts.data);
                      if (notice?.kind === 'auto') {
                        const acct = accounts.data?.find((a) => a.name === notice.accountName);
                        if (acct)
                          dispatch({
                            type: 'SET_ACCOUNT',
                            accountId: String(acct.id),
                            currency: acct.currency,
                          });
                      }
                      dispatch({
                        type: 'MAPPING_SUCCESS',
                        probe: res as UploadResponse,
                        originNotice: notice,
                      });
                      void qc.invalidateQueries({ queryKey: queryKeys.imports });
                    },
                  },
                );
              } else {
                // Resume mode: mapping already saved via putMapping, construct fresh probe
                const batchId = state.finalBatchId ?? state.probe!.batch_id;
                const freshProbe: UploadResponse = {
                  ...state.probe!,
                  batch_id: batchId,
                  mapping_used: columns,
                };
                dispatch({
                  type: 'MAPPING_SUCCESS',
                  probe: freshProbe,
                  originNotice: state.originNotice,
                });
                void qc.invalidateQueries({ queryKey: queryKeys.imports });
              }
            }}
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
            onBack={state.probe ? () => dispatch({ type: 'BACK_FROM_RECONCILE' }) : undefined}
            onCancel={onClose}
          />
        )}
      </WizardShell>
    </Dialog>
  );
}
