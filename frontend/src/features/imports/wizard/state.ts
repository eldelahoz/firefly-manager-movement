import type { UploadResponse } from '@/shared/api/dto';

export type WizardStep =
  'upload' | 'preview' | 'mapping' | 'reconcile-config' | 'submitting' | 'done' | 'error';

export type OriginNotice =
  { kind: 'auto'; accountName: string } | { kind: 'warn'; values: string[] } | null;

export interface WizardState {
  step: WizardStep;
  file: File | null;
  accountId: string;
  currency: string;
  probe: UploadResponse | null;
  finalBatchId: number | null;
  error: string | null;
  originNotice: OriginNotice;
}

export const initialWizardState: WizardState = {
  step: 'upload',
  file: null,
  accountId: '',
  currency: '',
  probe: null,
  finalBatchId: null,
  error: null,
  originNotice: null,
};

export type WizardAction =
  | { type: 'SET_ACCOUNT'; accountId: string; currency: string }
  | { type: 'UPLOAD_START'; file: File }
  | { type: 'UPLOAD_SUCCESS'; probe: UploadResponse; originNotice?: OriginNotice }
  | { type: 'UPLOAD_ERROR'; message: string }
  | { type: 'NEXT_FROM_PREVIEW' }
  | { type: 'MAPPING_SUCCESS'; probe: UploadResponse; originNotice?: OriginNotice }
  | { type: 'BACK_FROM_MAPPING' }
  | { type: 'BACK_FROM_RECONCILE' }
  | { type: 'BACK_FROM_PREVIEW' }
  | { type: 'REMOVE_FILE' }
  | { type: 'RECONCILE_START' }
  | { type: 'RECONCILE_SUCCESS'; batchId: number }
  | { type: 'RECONCILE_ERROR'; message: string }
  | { type: 'RESET' }
  | {
      type: 'RESUME_INIT';
      probe: UploadResponse;
      accountId: string;
      currency: string;
      batchId: number;
    };

export function wizardReducer(s: WizardState, a: WizardAction): WizardState {
  switch (a.type) {
    case 'SET_ACCOUNT':
      return { ...s, accountId: a.accountId, currency: a.currency };
    case 'UPLOAD_START':
      return { ...s, file: a.file, step: 'preview', probe: null, error: null };
    case 'UPLOAD_SUCCESS':
      return { ...s, step: 'preview', probe: a.probe, originNotice: a.originNotice ?? null };
    case 'UPLOAD_ERROR':
      return { ...s, step: 'upload', error: a.message, originNotice: null };
    case 'NEXT_FROM_PREVIEW':
      return { ...s, step: s.probe?.mapping_used ? 'reconcile-config' : 'mapping' };
    case 'MAPPING_SUCCESS':
      return {
        ...s,
        probe: a.probe,
        finalBatchId: a.probe.batch_id,
        step: 'reconcile-config',
        originNotice: a.originNotice ?? null,
      };
    case 'BACK_FROM_MAPPING':
      return { ...s, step: 'preview', error: null };
    case 'BACK_FROM_PREVIEW':
      // In resume mode (has finalBatchId), don't go back to upload
      if (s.finalBatchId != null) {
        return { ...s, step: 'reconcile-config', error: null };
      }
      return {
        ...s,
        step: 'upload',
        file: s.file,
        accountId: s.accountId,
        currency: s.currency,
        error: null,
        originNotice: null,
      };
    case 'REMOVE_FILE':
      return { ...s, file: null, probe: null, error: null, originNotice: null };
    case 'BACK_FROM_RECONCILE':
      return { ...s, step: s.file ? 'mapping' : 'preview', error: null };
    case 'RESUME_INIT':
      return {
        ...s,
        step: 'reconcile-config',
        probe: a.probe,
        accountId: a.accountId,
        currency: a.currency,
        finalBatchId: a.batchId,
      };
    case 'RECONCILE_START':
      return { ...s, step: 'submitting', error: null };
    case 'RECONCILE_SUCCESS':
      return { ...s, step: 'done', finalBatchId: a.batchId };
    case 'RECONCILE_ERROR':
      return { ...s, step: 'reconcile-config', error: a.message };
    case 'RESET':
      return initialWizardState;
  }
}
