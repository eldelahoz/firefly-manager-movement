import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { fixtures, handlers } from '@/test/handlers';
import { renderWithProviders } from '@/test/utils';
import { ImportWizard } from './ImportWizard';
import type { ImportDetail, UploadResponse } from '@/shared/api/dto';

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
  const input = document.querySelector('input[type="file"]') as HTMLInputElement;
  await userEvent.upload(input, file);
}

test('known mapping: preview then straight to reconcile config', async () => {
  server.use(
    http.get('/api/imports', () => HttpResponse.json([])),
    http.post('/api/imports', () => HttpResponse.json(knownMapping)),
  );
  renderWithProviders(<ImportWizard open onClose={() => {}} />);
  await upload(makeCsv());
  expect(
    await screen.findByText('auto-applied (saved mapping / auto-detected)'),
  ).toBeInTheDocument();
  await userEvent.click(screen.getByRole('button', { name: 'Next' }));
  expect(await screen.findByLabelText('Firefly III account')).toBeInTheDocument();
});

test('unknown mapping: preview then mapping step', async () => {
  server.use(
    http.get('/api/imports', () => HttpResponse.json([])),
    http.post('/api/imports', () => HttpResponse.json(probeBatch)),
  );
  renderWithProviders(<ImportWizard open onClose={() => {}} />);
  await upload(makeCsv());
  await screen.findByText(
    'We could not detect the columns automatically. Configure them in the next step to see the rows.',
  );
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

test('parse error shows banner and reset dropzone', async () => {
  server.use(
    http.get('/api/imports', () => HttpResponse.json([])),
    http.post('/api/imports', () =>
      HttpResponse.json({ error: 'Could not parse OFX: malformed header' }, { status: 400 }),
    ),
  );
  renderWithProviders(<ImportWizard open onClose={() => {}} />);
  await upload(makeCsv());
  expect(await screen.findByText('Could not parse OFX: malformed header')).toBeInTheDocument();
  expect(screen.getByText('Try another file?')).toBeInTheDocument();
});

test('duplicate filename is blocked before upload', async () => {
  let posted = false;
  let importsFetched = false;
  server.use(
    http.get('/api/imports', () => {
      importsFetched = true;
      return HttpResponse.json(fixtures.imports);
    }),
    http.post('/api/imports', () => {
      posted = true;
      return HttpResponse.json(knownMapping);
    }),
  );
  renderWithProviders(<ImportWizard open onClose={() => {}} />);
  await screen.findByLabelText('Firefly III account');
  await waitFor(() => expect(importsFetched).toBe(true));
  await upload(makeCsv()); // filename 'bank.csv' — already in fixtures.imports
  expect(await screen.findByText(/already exists/i)).toBeInTheDocument();
  expect(posted).toBe(false);
});

test('mapping confirm deletes probe batch before re-upload', async () => {
  const calls: string[] = [];
  server.use(
    http.get('/api/imports', () => HttpResponse.json([])),
    http.post('/api/imports', () => {
      calls.push('POST');
      return HttpResponse.json(probeBatch);
    }),
    http.put('/api/imports/:id/mapping', () => HttpResponse.json({ mapping_id: 1 })),
    http.delete('/api/imports/:id', ({ params }) => {
      calls.push(`DELETE:${params.id}`);
      return HttpResponse.json({ id: Number(params.id), status: 'deleted' });
    }),
  );
  renderWithProviders(<ImportWizard open onClose={() => {}} />);
  await upload(makeCsv());
  await userEvent.click(await screen.findByRole('button', { name: 'Next' })); // preview → mapping
  expect(await screen.findByLabelText('Date column')).toBeInTheDocument();
  await userEvent.click(screen.getByLabelText('Date column'));
  await userEvent.click(await screen.findByRole('option', { name: 'Date' }));
  await userEvent.click(screen.getByLabelText('Amount column'));
  await userEvent.click(await screen.findByRole('option', { name: 'Montant' }));
  await userEvent.click(screen.getByLabelText('Description column'));
  await userEvent.click(await screen.findByRole('option', { name: 'Libelle' }));
  await userEvent.click(screen.getByRole('button', { name: 'Next' })); // confirm mapping
  await waitFor(() => expect(calls).toEqual(['POST', 'DELETE:1', 'POST']));
});

test('drop fires upload and shows preview rows (parity with picker)', async () => {
  server.use(
    http.get('/api/imports', () => HttpResponse.json([])),
    http.post('/api/imports', () => HttpResponse.json(knownMapping)),
  );
  renderWithProviders(<ImportWizard open onClose={() => {}} />);
  await userEvent.click(await screen.findByLabelText('Firefly III account'));
  await userEvent.click(await screen.findByRole('option', { name: 'Checking (EUR)' }));
  const dropzone = screen
    .getByText('Drag CSV or OFX here')
    .closest('[role="button"]') as HTMLElement;
  fireEvent.drop(dropzone, { dataTransfer: { files: [makeCsv()] } });
  expect(
    await screen.findByText('auto-applied (saved mapping / auto-detected)'),
  ).toBeInTheDocument();
});

test('resumeBatchId initializes wizard at reconcile-config step', async () => {
  renderWithProviders(<ImportWizard open onClose={() => {}} resumeBatchId={3} />);
  expect(await screen.findByText('Firefly III account')).toBeInTheDocument();
  expect(await screen.findByRole('button', { name: 'Reconcile' })).toBeInTheDocument();
});

test('resume: back from reconcile-config navigates to preview (not close)', async () => {
  let closed = false;
  const resumeDetail: ImportDetail = {
    batch: {
      id: 5,
      filename: 'may.csv',
      file_hash: 'h5',
      format: 'csv',
      currency: 'EUR',
      status: 'parsed',
      created_at: '2026-08-20T10:00:00Z',
      account_id: 1,
      account_name: 'Checking',
      movements_count: 2,
    },
    movements: [
      {
        id: 20,
        line_no: 1,
        date: '2026-05-01',
        amount_cents: -1500,
        description: 'Rent',
        status: 'imported',
      },
      {
        id: 21,
        line_no: 2,
        date: '2026-05-03',
        amount_cents: 30000,
        description: 'Salary',
        status: 'imported',
      },
    ],
  };
  server.use(http.get('/api/imports/:id', () => HttpResponse.json(resumeDetail)));
  renderWithProviders(
    <ImportWizard
      open
      onClose={() => {
        closed = true;
      }}
      resumeBatchId={5}
    />,
  );
  expect(await screen.findByText('Firefly III account')).toBeInTheDocument();
  const backBtn = await screen.findByRole('button', { name: 'Back' });
  await userEvent.click(backBtn);
  expect(closed).toBe(false);
  expect(await screen.findByText('2 valid rows')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Next' })).toBeInTheDocument();
});

test('resume: back from reconcile-config with null mapping_used navigates to preview', async () => {
  let closed = false;
  const resumeDetailNoMapping: ImportDetail = {
    batch: {
      id: 6,
      filename: 'jun.csv',
      file_hash: 'h6',
      format: 'csv',
      currency: 'EUR',
      status: 'parsed',
      created_at: '2026-08-20T10:00:00Z',
      account_id: 1,
      account_name: 'Checking',
      movements_count: 1,
    },
    movements: [
      {
        id: 30,
        line_no: 1,
        date: '2026-06-01',
        amount_cents: -500,
        description: 'Groceries',
        status: 'imported',
      },
    ],
  };
  server.use(http.get('/api/imports/:id', () => HttpResponse.json(resumeDetailNoMapping)));
  renderWithProviders(
    <ImportWizard
      open
      onClose={() => {
        closed = true;
      }}
      resumeBatchId={6}
    />,
  );
  expect(await screen.findByText('Firefly III account')).toBeInTheDocument();
  const backBtn = await screen.findByRole('button', { name: 'Back' });
  await userEvent.click(backBtn);
  expect(closed).toBe(false);
  expect(await screen.findByText('1 valid rows')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Next' })).toBeInTheDocument();
});
