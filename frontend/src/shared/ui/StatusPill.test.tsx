import { render, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { StatusPill, type PillStatus } from './StatusPill';

test.each([
  ['imported', 'Imported'],
  ['matched', 'Matched'],
  ['missing', 'Missing'],
  ['created', 'Created'],
  ['ignored', 'Ignored'],
  ['duplicate', 'Duplicate'],
  ['already_created', 'Already created'],
  ['failed', 'Failed'],
])('status %s renders label %s', (rawStatus, label) => {
  const status = rawStatus as PillStatus;
  render(<StatusPill status={status} />);
  expect(screen.getByText(label)).toBeInTheDocument();
});

test('matched uses green tokens', () => {
  render(<StatusPill status="matched" />);
  const pill = screen.getByText('Matched').parentElement;
  expect(pill).toHaveClass('bg-status-matched-bg', 'text-status-matched-txt');
});

test('imported uses solid neutral background without border', () => {
  render(<StatusPill status="imported" />);
  const pill = screen.getByText('Imported').parentElement;
  expect(pill).toHaveClass('bg-border-subtle');
  expect([...pill!.classList]).not.toContain('border');
});

test('duplicate is filled red-soft without border', () => {
  render(<StatusPill status="duplicate" />);
  const pill = screen.getByText('Duplicate').parentElement;
  expect(pill).toHaveClass('bg-status-extra-bg');
  expect([...pill!.classList].some((c) => c.startsWith('border-'))).toBe(false);
});

test('labels render uppercase via CSS while DOM text stays original', () => {
  render(<StatusPill status="matched" />);
  expect(screen.getByText('Matched').parentElement).toHaveClass('uppercase');
});
