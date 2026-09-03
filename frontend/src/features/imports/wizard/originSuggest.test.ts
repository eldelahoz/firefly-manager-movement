import { expect, test } from 'vitest';
import type { Ff3Account } from '@/shared/api/dto';
import { suggestAccount } from './originSuggest';

const accounts: Ff3Account[] = [
  {
    id: 1,
    name: 'Checking',
    type: 'asset',
    currency: 'EUR',
    iban: 'DE44 5001 0517 5407 3249 31',
    account_number: '1001',
  },
  {
    id: 2,
    name: 'Savings',
    type: 'asset',
    currency: 'EUR',
    iban: null,
    account_number: null,
  },
];

test('unique IBAN match with spaces and case differences', () => {
  expect(suggestAccount(['de44 5001 0517 5407 3249 31'], accounts).account?.id).toBe(1);
});

test('match by account number or name', () => {
  expect(suggestAccount(['1001'], accounts).account?.id).toBe(1);
  expect(suggestAccount(['savings'], accounts).account?.id).toBe(2);
});

test('single value matching one account pre-selects it', () => {
  expect(suggestAccount(['DE44 5001 0517 5407 3249 31'], accounts)).toEqual(
    expect.objectContaining({
      account: expect.objectContaining({ id: 1 }),
      warning: false,
    }),
  );
});

test('several distinct values keep manual selection with warning', () => {
  const s = suggestAccount(['DE44 5001 0517 5407 3249 31', '2002'], accounts);
  expect(s.account).toBeNull();
  expect(s.warning).toBe(true);
  expect(s.values).toEqual(['DE44 5001 0517 5407 3249 31', '2002']);
});

test('single value matching no account warns listing the value', () => {
  const s = suggestAccount(['unknown'], accounts);
  expect(s.account).toBeNull();
  expect(s.warning).toBe(true);
});

test('no origin values: no suggestion, no warning', () => {
  const s = suggestAccount([], accounts);
  expect(s.account).toBeNull();
  expect(s.warning).toBe(false);
});
