import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { ImportWizard } from './ImportWizard';
import type { UploadResponse } from '@/shared/api/dto';

const server = setupServer(...handlers);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

const knownMapping: UploadResponse = {
  batch_id: 3,
  filename: 'bank.csv',
  file_hash: 'h',
  headers: ['Date', 'Montant', 'Libelle'],
  detected_delimiter: ';',
  total_rows: 2,
  valid_count: 2,
  invalid_count: 0,
  mapping_used: { date_column: 'Date', amount_column: 'Montant', description_column: 'Libelle' },
  valid: [{ line_no: 1, date: '2026-08-01', amount_cents: -4200, description: 'Coffee' }],
  invalid: [],
};

const probeBatch: UploadResponse = {
  ...knownMapping,
  batch_id: 1,
  mapping_used: null,
  valid: [],
  valid_count: 0,
};

function makeCsv() {
  return new File(['Date;Montant\n2026-08-01;-42\n'], 'bank.csv', { type: 'text/csv' });
}

async function upload(file: File) {
  await userEvent.click(await screen.findByLabelText('Firefly III account'));
  await userEvent.click(await screen.findByRole('option', { name: 'Checking (EUR)' }));
  const drop = screen.getByText('Drop your CSV here or click to browse');
  const input = drop.parentElement!.parentElement!.querySelector(
    'input[type="file"]',
  ) as HTMLInputElement;
  await userEvent.upload(input, file);
}

test('known mapping: preview then straight to reconcile config', async () => {
  server.use(http.post('/api/imports', () => HttpResponse.json(knownMapping)));
  renderWithProviders(<ImportWizard open onClose={() => {}} />);
  await upload(makeCsv());
  expect(
    await screen.findByText('auto-applied (saved mapping / auto-detected)'),
  ).toBeInTheDocument();
  await userEvent.click(screen.getByRole('button', { name: 'Next' }));
  expect(await screen.findByLabelText('Firefly III account')).toBeInTheDocument();
});

test('unknown mapping: preview then mapping step', async () => {
  server.use(http.post('/api/imports', () => HttpResponse.json(probeBatch)));
  renderWithProviders(<ImportWizard open onClose={() => {}} />);
  await upload(makeCsv());
  await screen.findByRole('button', { name: 'Next' });
  await userEvent.click(screen.getByRole('button', { name: 'Next' }));
  expect(await screen.findByLabelText('Date column')).toBeInTheDocument();
});

test('oversize file shows inline error without upload', async () => {
  renderWithProviders(<ImportWizard open onClose={() => {}} />);
  const big = new File([new ArrayBuffer(10 * 1024 * 1024 + 1)], 'big.csv');
  await upload(big);
  expect(await screen.findByText('File too large (max 10 MB)')).toBeInTheDocument();
});
