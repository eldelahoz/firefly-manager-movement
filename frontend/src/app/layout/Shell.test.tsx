import { screen } from '@testing-library/react';
import { Shell } from './Shell';
import { renderWithProviders } from '@/test/utils';

test('shell renders nav translated with settings link', () => {
  renderWithProviders(<Shell />);
  expect(screen.getAllByText('Dashboard').length).toBeGreaterThan(0);
  expect(screen.getAllByText('Imports').length).toBeGreaterThan(0);
  expect(screen.getAllByRole('link', { name: 'Settings' }).length).toBeGreaterThan(0);
});
