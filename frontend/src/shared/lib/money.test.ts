import { describe, expect, test } from 'vitest';
import { formatMoney, formatSignedMoney } from './money';

describe('formatMoney', () => {
  test('formats cents as currency for en-US', () => {
    expect(formatMoney(123456, 'EUR', 'en-US')).toBe('€1,234.56');
  });
  test('negative amounts keep sign', () => {
    expect(formatMoney(-990, 'EUR', 'en-US')).toBe('-€9.90');
  });
  test('zero', () => {
    expect(formatMoney(0, 'EUR', 'en-US')).toBe('€0.00');
  });
  test('fr locale contains euro sign and comma decimals', () => {
    const s = formatMoney(123456, 'EUR', 'fr-FR');
    expect(s).toContain('€');
    expect(s).toContain(',');
  });
});

describe('formatSignedMoney', () => {
  test('positive gets plus', () => {
    expect(formatSignedMoney(500, 'EUR', 'en-US')).toBe('+€5.00');
  });
  test('negative gets minus sign', () => {
    expect(formatSignedMoney(-500, 'EUR', 'en-US')).toBe('−€5.00');
  });
  test('zero is neutral', () => {
    expect(formatSignedMoney(0, 'EUR', 'en-US')).toBe('€0.00');
  });
});
