import { useTranslation } from 'react-i18next';
import { useTheme } from '@/app/ThemeProvider';
import { Button } from '@/shared/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';

const LANGUAGES = [
  { value: 'en', label: 'English' },
  { value: 'es', label: 'Español' },
  { value: 'fr', label: 'Français' },
] as const;

export function SettingsPage() {
  const { t, i18n } = useTranslation();
  const { theme, setTheme } = useTheme();
  const current = i18n.resolvedLanguage ?? i18n.language;
  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">{t('settings.title')}</h1>
      <section className="rounded-xl border border-border-subtle bg-surface p-4">
        <h2 className="font-semibold">{t('settings.appearance')}</h2>
        <div className="mt-3 flex gap-2">
          <Button
            variant={theme === 'light' ? 'default' : 'outline'}
            onClick={() => setTheme('light')}
          >
            {t('settings.themeLight')}
          </Button>
          <Button
            variant={theme === 'dark' ? 'default' : 'outline'}
            onClick={() => setTheme('dark')}
          >
            {t('settings.themeDark')}
          </Button>
        </div>
      </section>
      <section className="rounded-xl border border-border-subtle bg-surface p-4">
        <h2 className="font-semibold">{t('settings.language')}</h2>
        <Select
          value={current}
          onValueChange={(v) => {
            void i18n.changeLanguage(v);
          }}
        >
          <SelectTrigger className="mt-3 w-[180px]" aria-label={t('settings.language')}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent position="popper">
            {LANGUAGES.map((l) => (
              <SelectItem key={l.value} value={l.value}>
                {l.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </section>
    </div>
  );
}
