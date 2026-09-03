import { expect, test } from 'vitest';
import { applyBulkFillOnly } from './customization';

test('fill-only: empty fields leave per-movement values untouched', () => {
  const current = { 1: { tags: ['x'], category: 'Food' }, 2: { counterpartyId: 9 } };
  const next = applyBulkFillOnly(current, [1, 2], { category: 'Bills' });
  expect(next[1]).toEqual({ tags: ['x'], category: 'Bills' });
  expect(next[2]).toEqual({ counterpartyId: 9, category: 'Bills' });
});

test('undefined keys are not applied', () => {
  const next = applyBulkFillOnly({ 1: { description: 'orig' } }, [1], { description: undefined });
  expect(next[1].description).toBe('orig');
});

test('empty string is an explicit clear', () => {
  const next = applyBulkFillOnly({ 1: { notes: 'n' } }, [1], { notes: '' });
  expect(next[1].notes).toBe('');
});
