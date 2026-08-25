import { render, screen } from '@testing-library/react';
import { expect, test } from 'vitest';
import { EmptyState } from './EmptyState';

test('renders title and action', () => {
  render(<EmptyState title="Nothing here" action={<button>Go</button>} />);
  expect(screen.getByText('Nothing here')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Go' })).toBeInTheDocument();
});
