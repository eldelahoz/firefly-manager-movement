import { expect, test } from 'vitest';
import { render, screen } from '@testing-library/react';
import { PageHeader } from './PageHeader';

test('renders title, description and actions', () => {
  render(
    <PageHeader title="Imports" description="Manage batches" actions={<button>Do it</button>} />,
  );
  expect(screen.getByRole('heading', { level: 1, name: 'Imports' })).toBeInTheDocument();
  expect(screen.getByText('Manage batches')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Do it' })).toBeInTheDocument();
});

test('renders title only', () => {
  render(<PageHeader title="Dashboard" />);
  expect(screen.getByRole('heading', { level: 1, name: 'Dashboard' })).toBeInTheDocument();
});
