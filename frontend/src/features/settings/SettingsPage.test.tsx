import { afterEach, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { renderWithProviders } from '@/test/utils';
import { ThemeProvider } from '@/app/ThemeProvider';
import i18n from '@/shared/i18n';
import { SettingsPage } from './SettingsPage';

function renderSettings() {
  return renderWithProviders(
    <ThemeProvider>
      <SettingsPage />
    </ThemeProvider>,
  );
}

afterEach(() => {
  document.documentElement.classList.remove('dark');
  localStorage.removeItem('ffm.theme');
  void i18n.changeLanguage('en');
});

test('renders appearance and language sections', () => {
  renderSettings();
  expect(screen.getByRole('heading', { name: 'Settings' })).toBeInTheDocument();
  expect(screen.getByText('Appearance')).toBeInTheDocument();
  expect(screen.getByText('Language')).toBeInTheDocument();
});

test('selecting dark theme applies dark class', async () => {
  renderSettings();
  await userEvent.click(screen.getByRole('button', { name: 'Dark' }));
  expect(document.documentElement).toHaveClass('dark');
});

test('language select switches to Spanish', async () => {
  renderSettings();
  await userEvent.click(screen.getByRole('combobox'));
  await userEvent.click(await screen.findByRole('option', { name: 'Español' }));
  expect(i18n.language).toBe('es');
  expect(screen.getByRole('heading', { name: 'Ajustes' })).toBeInTheDocument();
});
