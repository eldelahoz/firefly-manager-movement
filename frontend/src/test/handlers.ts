import { http, HttpResponse } from 'msw';

export const fixtures = {
  accounts: [
    { id: 1, name: 'Checking', type: 'asset', currency: 'EUR' },
    { id: 2, name: 'Savings USD', type: 'asset', currency: 'USD' },
  ],
  imports: [
    {
      id: 1,
      filename: 'bank.csv',
      file_hash: 'h1',
      format: 'csv',
      currency: 'EUR',
      status: 'reconciled',
      created_at: '2026-08-20T10:00:00Z',
    },
  ],
  results: {
    reconciliation_id: 1,
    matched: [
      {
        movement: {
          id: 11,
          line_no: 2,
          date: '2026-08-01',
          amount_cents: -4200,
          description: 'Coffee',
          status: 'matched',
        },
        ff3: {
          id: 7,
          ff3_id: 42,
          date: '2026-08-01',
          amount_cents: -4200,
          description: 'Coffee',
          is_duplicate: false,
          created_at: '2026-08-01T00:00:00Z',
        },
      },
    ],
    missing: [
      {
        id: 12,
        line_no: 3,
        date: '2026-08-02',
        amount_cents: -9900,
        description: 'Book',
        status: 'missing',
      },
      {
        id: 13,
        line_no: 4,
        date: '2026-08-05',
        amount_cents: 250000,
        description: 'Salary',
        status: 'missing',
      },
    ],
    extra: [
      {
        id: 8,
        ff3_id: 43,
        date: '2026-08-03',
        amount_cents: -1500,
        description: 'Duplicate A',
        is_duplicate: true,
        created_at: '2026-08-03T00:00:00Z',
      },
    ],
  },
  txs: [
    { id: 42, date: '2026-08-01', amount_cents: -4200, description: 'Coffee' },
    { id: 43, date: '2026-08-03', amount_cents: -1500, description: 'Duplicate A' },
  ],
};

export const handlers = [
  http.get('/health', () => HttpResponse.json({ status: 'ok' })),
  http.get('/api/imports', () => HttpResponse.json(fixtures.imports)),
  http.get('/api/imports/:id', () =>
    HttpResponse.json({ batch: fixtures.imports[0], movements: fixtures.results.missing }),
  ),
  http.get('/api/imports/:id/results', () => HttpResponse.json(fixtures.results)),
  http.get('/api/firefly/accounts', () => HttpResponse.json(fixtures.accounts)),
  http.post('/api/firefly/accounts/:id/transactions', () => HttpResponse.json(fixtures.txs)),
  http.delete('/api/firefly/transactions/:id', () =>
    HttpResponse.json({ ff3_id: 42, status: 'deleted' }),
  ),
  http.post('/api/imports/:id/movements/:mid/ignore', ({ params }) =>
    HttpResponse.json({ movement_id: Number(params.mid), status: 'ignored' }),
  ),
];
