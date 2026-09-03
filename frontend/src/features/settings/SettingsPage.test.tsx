import { afterAll, afterEach, beforeAll, expect, test } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';
import { Toaster } from 'sonner';
import { renderWithProviders } from '@/test/utils';
import { ThemeProvider } from '@/app/ThemeProvider';
import i18n from '@/shared/i18n';
import { SettingsPage } from './SettingsPage';

const server = setupServer(
  http.get('/api/settings', () =>
    HttpResponse.json({
      ff3_url: 'http://localhost:8080',
      has_token: true,
      default_creation_tag: 'ffm-import',
    }),
  ),
  http.get('/api/firefly/tags', () => HttpResponse.json(['ffm-import', 'vacation'])),
);
beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

function renderSettings() {
  return renderWithProviders(
    <ThemeProvider>
      <Toaster />
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
  await userEvent.click(screen.getByRole('combobox', { name: 'Language' }));
  await userEvent.click(await screen.findByRole('option', { name: 'Español' }));
  expect(i18n.language).toBe('es');
  expect(screen.getByRole('heading', { name: 'Ajustes' })).toBeInTheDocument();
});

test('renders connection section with URL, token, default tag dropdown listing FF3 tags', async () => {
  renderSettings();
  expect(await screen.findByText('Firefly III connection')).toBeInTheDocument();
  expect(await screen.findByLabelText('Firefly III URL')).toHaveValue('http://localhost:8080');
  // token never echoed: input starts empty
  expect(screen.getByLabelText('API token')).toHaveValue('');
  await userEvent.click(screen.getByRole('combobox', { name: 'Default creation tag' }));
  expect(await screen.findByRole('option', { name: 'ffm-import' })).toBeInTheDocument();
  expect(screen.getByRole('option', { name: 'No default tag' })).toBeInTheDocument();
});

test('saving persists url + default tag and shows success', async () => {
  let putBody: unknown = null;
  server.use(
    http.put('/api/settings', async ({ request }) => {
      putBody = await request.json();
      return HttpResponse.json({
        ff3_url: 'http://localhost:8080',
        has_token: true,
        default_creation_tag: null,
      });
    }),
  );
  renderSettings();
  await screen.findByText('Firefly III connection');
  // change tag to "none"
  await userEvent.click(screen.getByRole('combobox', { name: 'Default creation tag' }));
  await userEvent.click(await screen.findByRole('option', { name: 'No default tag' }));
  await userEvent.click(screen.getByRole('button', { name: 'Save' }));
  await screen.findByText('Settings saved');
  expect(putBody).toMatchObject({
    ff3_url: 'http://localhost:8080',
    default_creation_tag: null,
  });
});

test('test connection reports the FF3 version', async () => {
  server.use(http.post('/api/settings/verify', () => HttpResponse.json({ version: '6.1.36' })));
  renderSettings();
  await screen.findByText('Firefly III connection');
  await userEvent.type(screen.getByLabelText('Firefly III URL'), 'http://ff3');
  await userEvent.type(screen.getByLabelText('API token'), 'tok');
  await userEvent.click(screen.getByRole('button', { name: 'Test connection' }));
  expect(await screen.findByText(/Connection OK/)).toBeInTheDocument();
});
