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
      // Stage the file only: stay on upload; the parse runs on explicit Next (RF-01 v0.1)
      return { ...s, file: a.file, probe: null, error: null, originNotice: null };
    case 'UPLOAD_SUCCESS':
      // A usable mapping (saved signature / auto-detect) lands on the preview to confirm it;
      // otherwise the user maps columns first and the preview shows post-mapping rows.
      return {
        ...s,
        step: a.probe.mapping_used && a.probe.valid_count > 0 ? 'preview' : 'mapping',
        probe: a.probe,
        originNotice: a.originNotice ?? null,
      };
    case 'UPLOAD_ERROR':
      return { ...s, step: 'upload', error: a.message, originNotice: null };
    case 'NEXT_FROM_PREVIEW':
      return { ...s, step: s.probe?.mapping_used ? 'reconcile-config' : 'mapping' };
    case 'MAPPING_SUCCESS':
      // Re-upload with the chosen columns already ran: show the parsed result.
      return {
        ...s,
        probe: a.probe,
        finalBatchId: a.probe.batch_id,
        step: 'preview',
        originNotice: a.originNotice ?? null,
      };
    case 'BACK_FROM_MAPPING':
      // Upload only makes sense with a staged file; resume mode loops back to preview.
      return { ...s, step: s.file ? 'upload' : 'preview', error: null };
    case 'BACK_FROM_PREVIEW':
      // Previous step is mapping in every path (auto-applied, manual, resume);
      // mapping seeds the current mapping_used, upload stays one Back further.
      return { ...s, step: 'mapping', error: null };
    case 'REMOVE_FILE':
      return { ...s, file: null, probe: null, error: null, originNotice: null };
    case 'BACK_FROM_RECONCILE':
      return { ...s, step: 'preview', error: null };
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
