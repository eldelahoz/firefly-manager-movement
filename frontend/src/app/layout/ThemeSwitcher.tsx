import { Moon, Sun } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useTheme } from '../ThemeProvider';

export function ThemeSwitcher() {
  const { t } = useTranslation();
  const { theme, toggle } = useTheme();
  return (
    <button
      onClick={toggle}
      aria-label={theme === 'dark' ? t('nav.themeToLight') : t('nav.themeToDark')}
      className="flex size-9 items-center justify-center self-start rounded-lg text-text-2 hover:bg-canvas"
    >
      {theme === 'dark' ? <Sun className="size-5" /> : <Moon className="size-5" />}
    </button>
  );
}
