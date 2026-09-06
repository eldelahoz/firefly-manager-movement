import { http, HttpResponse } from 'msw';

export const fixtures = {
  accounts: [
    {
      id: 1,
      name: 'Checking',
      type: 'asset',
      currency: 'EUR',
      iban: 'DE44 5001 0517 5407 3249 31',
      account_number: null,
    },
    {
      id: 2,
      name: 'Savings USD',
      type: 'asset',
      currency: 'USD',
      iban: null,
      account_number: null,
    },
  ],
  expenseAccounts: [
    {
      id: 51,
      name: 'Supermarket',
      type: 'expense',
      currency: 'EUR',
      iban: null,
      account_number: null,
    },
  ],
  revenueAccounts: [
    {
      id: 61,
      name: 'Employer',
      type: 'revenue',
      currency: 'EUR',
      iban: null,
      account_number: null,
    },
  ],
  settings: {
    ff3_url: 'http://localhost:8080',
    has_token: true,
    default_creation_tag: 'ffm-import',
    enable_origin_trace: true,
  },
  tags: ['ffm-import', 'vacation'],
  categories: ['Groceries'],
  budgets: ['Daily'],
  imports: [
    {
      id: 1,
      filename: 'bank.csv',
      file_hash: 'h1',
      format: 'csv',
      currency: 'EUR',
      status: 'reconciled',
      created_at: '2026-08-20T10:00:00Z',
      account_id: null,
      account_name: null,
      movements_count: 2,
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
    ignored: [
      {
        id: 13,
        line_no: 4,
        date: '2026-08-02',
        amount_cents: 1500,
        description: 'Internal transfer',
        status: 'ignored',
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
    {
      id: 42,
      date: '2026-08-01',
      amount_cents: -4200,
      description: 'Coffee',
      status: 'matched',
    },
    {
      id: 43,
      date: '2026-08-03',
      amount_cents: -1500,
      description: 'Duplicate A',
      status: 'duplicate',
    },
    {
      id: 44,
      date: '2026-08-05',
      amount_cents: 250000,
      description: 'Salary',
      status: null,
    },
  ],
};

function accountsForType(type?: string | null) {
  if (!type || type === 'asset') return fixtures.accounts;
  if (type === 'expense') return fixtures.expenseAccounts;
  if (type === 'revenue') return fixtures.revenueAccounts;
  return [];
}

export const handlers = [
  http.get('/health', () => HttpResponse.json({ status: 'ok' })),
  http.get('/api/imports', () => HttpResponse.json(fixtures.imports)),
  http.get('/api/imports/:id', () =>
    HttpResponse.json({ batch: fixtures.imports[0], movements: fixtures.results.missing }),
  ),
  http.get('/api/imports/:id/results', () => HttpResponse.json(fixtures.results)),
  http.get('/api/firefly/accounts', ({ request }) => {
    const url = new URL(request.url);
    const type = url.searchParams.get('type');
    return HttpResponse.json(accountsForType(type));
  }),
  http.post('/api/firefly/accounts/:id/transactions', () => HttpResponse.json(fixtures.txs)),
  http.delete('/api/firefly/transactions/:id', () =>
    HttpResponse.json({ ff3_id: 42, status: 'deleted' }),
  ),
  http.post('/api/imports/:id/movements/:mid/ignore', ({ params }) =>
    HttpResponse.json({ movement_id: Number(params.mid), status: 'ignored' }),
  ),
  http.post('/api/imports/:id/apply', () =>
    HttpResponse.json({
      reports: [],
      created_count: 0,
      already_created_count: 0,
      failed_count: 0,
    }),
  ),
  http.delete('/api/imports/:id', ({ params }) =>
    HttpResponse.json({ id: Number(params.id), status: 'deleted' }),
  ),
  http.get('/api/settings', () => HttpResponse.json(fixtures.settings)),
  http.put('/api/settings', () => HttpResponse.json(fixtures.settings)),
  http.post('/api/settings/verify', () => HttpResponse.json({ version: '6.1.36' })),
  http.get('/api/firefly/tags', () => HttpResponse.json(fixtures.tags)),
  http.get('/api/firefly/categories', () => HttpResponse.json(fixtures.categories)),
  http.get('/api/firefly/budgets', () => HttpResponse.json(fixtures.budgets)),
];
