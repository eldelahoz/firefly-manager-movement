import { render, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { StatusPill } from './StatusPill';

test.each([
  ['imported', 'Imported'],
  ['matched', 'Matched'],
  ['missing', 'Missing'],
  ['created', 'Created'],
  ['ignored', 'Ignored'],
  ['duplicate', 'Duplicate'],
  ['already_created', 'Already created'],
  ['failed', 'Failed'],
])('status %s renders label %s', (status, label) => {
  render(<StatusPill status={status} />);
  expect(screen.getByText(label)).toBeInTheDocument();
});

test('matched uses green tokens', () => {
  render(<StatusPill status="matched" />);
  expect(screen.getByText('Matched')).toHaveClass('bg-status-matched-bg', 'text-status-matched-txt');
});
