import type { UploadResponse } from '@/shared/api/dto';

export type WizardStep =
  'upload' | 'preview' | 'mapping' | 'reconcile-config' | 'submitting' | 'done' | 'error';

export interface WizardState {
  step: WizardStep;
  file: File | null;
  accountId: string;
  currency: string;
  probe: UploadResponse | null;
  finalBatchId: number | null;
  error: string | null;
}

export const initialWizardState: WizardState = {
  step: 'upload',
  file: null,
  accountId: '',
  currency: '',
  probe: null,
  finalBatchId: null,
  error: null,
};

export type WizardAction =
  | { type: 'SET_ACCOUNT'; accountId: string; currency: string }
  | { type: 'UPLOAD_START'; file: File }
  | { type: 'UPLOAD_SUCCESS'; probe: UploadResponse }
  | { type: 'UPLOAD_ERROR'; message: string }
  | { type: 'NEXT_FROM_PREVIEW' }
  | { type: 'MAPPING_SUCCESS'; probe: UploadResponse }
  | { type: 'RECONCILE_START' }
  | { type: 'RECONCILE_SUCCESS'; batchId: number }
  | { type: 'RECONCILE_ERROR'; message: string }
  | { type: 'RESET' };

export function wizardReducer(s: WizardState, a: WizardAction): WizardState {
  switch (a.type) {
    case 'SET_ACCOUNT':
      return { ...s, accountId: a.accountId, currency: a.currency };
    case 'UPLOAD_START':
      return { ...s, file: a.file, step: 'preview', probe: null, error: null };
    case 'UPLOAD_SUCCESS':
      return { ...s, step: 'preview', probe: a.probe };
    case 'UPLOAD_ERROR':
      return { ...s, step: 'upload', error: a.message };
    case 'NEXT_FROM_PREVIEW':
      return { ...s, step: s.probe?.mapping_used ? 'reconcile-config' : 'mapping' };
    case 'MAPPING_SUCCESS':
      return { ...s, probe: a.probe, finalBatchId: a.probe.batch_id, step: 'reconcile-config' };
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
