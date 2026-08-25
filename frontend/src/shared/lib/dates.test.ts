import { describe, expect, test } from 'vitest';
import { addDays, formatDate, toIsoDate } from './dates';

describe('toIsoDate', () => {
  test('formats YYYY-MM-DD', () => {
    expect(toIsoDate(new Date(Date.UTC(2026, 7, 24)))).toBe('2026-08-24');
  });
});

describe('addDays', () => {
  test('adds days across month boundary', () => {
    expect(addDays('2026-08-30', 3)).toBe('2026-09-02');
  });
  test('negative days', () => {
    expect(addDays('2026-09-02', -3)).toBe('2026-08-30');
  });
});

describe('formatDate', () => {
  test('en medium style', () => {
    expect(formatDate('2026-08-24', 'en-US')).toBe('Aug 24, 2026');
  });
});
