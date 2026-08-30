import { screen } from '@testing-library/react';
import { Shell } from './Shell';
import { renderWithProviders } from '@/test/utils';

test('shell renders nav, aside footer with health and theme, and no header', () => {
  renderWithProviders(<Shell />);
  expect(screen.getAllByText('Dashboard').length).toBeGreaterThan(0);
  expect(screen.getAllByText('Imports').length).toBeGreaterThan(0);
  expect(screen.getAllByRole('link', { name: 'Settings' }).length).toBeGreaterThan(0);
  expect(screen.getByText('Health unknown')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Switch to dark theme' })).toBeInTheDocument();
  expect(screen.queryByRole('banner')).not.toBeInTheDocument();
});
