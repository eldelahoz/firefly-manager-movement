import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { renderWithProviders } from '@/test/utils';
import { MappingStep } from './MappingStep';
import type { UploadResponse } from '@/shared/api/dto';

const server = setupServer();
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

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
  valid: [],
  invalid: [],
};

test('next disabled until all columns chosen; PUT mapping then onMapped', async () => {
  let putBody: unknown = null;
  server.use(
    http.put('/api/imports/1/mapping', async ({ request }) => {
      putBody = await request.json();
      return HttpResponse.json({ mapping_id: 5 });
    }),
  );
  let mapped: unknown = null;
  renderWithProviders(<MappingStep probe={probe} onMapped={(c) => (mapped = c)} />);
  expect(screen.getByRole('button', { name: 'Next' })).toBeDisabled();
  for (const [label, header] of [
    ['Date column', 'Date'],
    ['Amount column', 'Montant'],
    ['Description column', 'Libelle'],
  ] as const) {
    await userEvent.click(screen.getByLabelText(label));
    await userEvent.click(await screen.findByRole('option', { name: header, exact: true }));
  }
  await userEvent.click(screen.getByRole('button', { name: 'Next' }));
  expect(putBody).toEqual({
    headers: ['Date', 'Montant', 'Libelle'],
    date_column: 'Date',
    amount_column: 'Montant',
    description_column: 'Libelle',
  });
  expect(mapped).toEqual({
    date_column: 'Date',
    amount_column: 'Montant',
    description_column: 'Libelle',
  });
});
