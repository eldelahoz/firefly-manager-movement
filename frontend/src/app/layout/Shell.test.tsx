import { screen } from '@testing-library/react';
import { Shell } from './Shell';
import { renderWithProviders } from '@/test/utils';

test('shell renders nav translated and settings disabled', () => {
  renderWithProviders(<Shell />);
  expect(screen.getAllByText('Dashboard').length).toBeGreaterThan(0);
  expect(screen.getAllByText('Imports').length).toBeGreaterThan(0);
  expect(screen.getAllByText('Settings')[0].closest('[aria-disabled]')).toHaveAttribute(
    'aria-disabled',
    'true',
  );
});
