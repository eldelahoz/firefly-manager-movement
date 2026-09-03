import { expect, test } from 'vitest';
import { initialWizardState, wizardReducer } from './state';
import type { UploadResponse } from '@/shared/api/dto';

const probe: UploadResponse = {
  batch_id: 1,
  filename: 'bank.csv',
  file_hash: 'h',
  headers: ['Date', 'Montant', 'Libelle'],
  detected_delimiter: ';',
  total_rows: 0,
  valid_count: 0,
  invalid_count: 0,
  mapping_used: null,
  origin_account_values: [],
  valid: [],
  invalid: [],
};

test('upload success with mapping lands on preview with probe stored', () => {
  const s = wizardReducer(initialWizardState, {
    type: 'UPLOAD_SUCCESS',
    probe: {
      ...probe,
      mapping_used: { date_column: 'a', amount_column: 'b', description_column: 'c' },
    },
  });
  expect(s.step).toBe('preview');
});

test('next from preview skips mapping when mapping known', () => {
  const s1 = wizardReducer(initialWizardState, {
    type: 'UPLOAD_SUCCESS',
    probe: {
      ...probe,
      mapping_used: { date_column: 'a', amount_column: 'b', description_column: 'c' },
    },
  });
  const s2 = wizardReducer(s1, { type: 'NEXT_FROM_PREVIEW' });
  expect(s2.step).toBe('reconcile-config');
});

test('next from preview enters mapping when mapping null', () => {
  const s1 = wizardReducer(initialWizardState, { type: 'UPLOAD_SUCCESS', probe });
  const s2 = wizardReducer(s1, { type: 'NEXT_FROM_PREVIEW' });
  expect(s2.step).toBe('mapping');
});

test('mapping success stores final batch and moves to reconcile-config', () => {
  const s1 = wizardReducer(initialWizardState, { type: 'UPLOAD_SUCCESS', probe });
  const s2 = wizardReducer(s1, { type: 'NEXT_FROM_PREVIEW' });
  const s3 = wizardReducer(s2, {
    type: 'MAPPING_SUCCESS',
    probe: {
      ...probe,
      batch_id: 2,
      valid_count: 5,
      mapping_used: {
        date_column: 'Date',
        amount_column: 'Montant',
        description_column: 'Libelle',
      },
    },
  });
  expect(s3.step).toBe('reconcile-config');
  expect(s3.finalBatchId).toBe(2);
});

test('reconcile start then success reaches done', () => {
  const s = wizardReducer(initialWizardState, { type: 'RECONCILE_START' });
  expect(s.step).toBe('submitting');
  const s2 = wizardReducer(s, { type: 'RECONCILE_SUCCESS', batchId: 9 });
  expect(s2.step).toBe('done');
  expect(s2.finalBatchId).toBe(9);
});

test('reconcile error returns to config with error set', () => {
  const s = wizardReducer(initialWizardState, { type: 'RECONCILE_START' });
  const s2 = wizardReducer(s, { type: 'RECONCILE_ERROR', message: 'currency mismatch' });
  expect(s2.step).toBe('reconcile-config');
  expect(s2.error).toBe('currency mismatch');
});

test('set account stores account and derived currency', () => {
  const s = wizardReducer(initialWizardState, {
    type: 'SET_ACCOUNT',
    accountId: '2',
    currency: 'USD',
  });
  expect(s.accountId).toBe('2');
  expect(s.currency).toBe('USD');
});

test('back from reconcile returns to mapping keeping probe', () => {
  const s0 = wizardReducer(initialWizardState, {
    type: 'UPLOAD_START',
    file: new File([''], 'test.csv'),
  });
  const s1 = wizardReducer(s0, { type: 'UPLOAD_SUCCESS', probe });
  const s2 = wizardReducer(s1, { type: 'NEXT_FROM_PREVIEW' });
  const s3 = wizardReducer(s2, { type: 'MAPPING_SUCCESS', probe: { ...probe, batch_id: 2 } });
  const s4 = wizardReducer(s3, { type: 'BACK_FROM_RECONCILE' });
  expect(s4.step).toBe('mapping');
  expect(s4.probe).not.toBeNull();
});

test('back from mapping returns to preview', () => {
  const s1 = wizardReducer(initialWizardState, { type: 'UPLOAD_SUCCESS', probe });
  const s2 = wizardReducer(s1, { type: 'NEXT_FROM_PREVIEW' });
  const s3 = wizardReducer(s2, { type: 'BACK_FROM_MAPPING' });
  expect(s3.step).toBe('preview');
  expect(s3.probe).not.toBeNull();
});

test('reset returns to initial', () => {
  const s = wizardReducer(initialWizardState, { type: 'RESET' });
  expect(s).toEqual(initialWizardState);
});
