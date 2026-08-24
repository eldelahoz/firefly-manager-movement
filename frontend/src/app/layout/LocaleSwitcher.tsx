import { useTranslation } from 'react-i18next';

const LOCALES = ['en', 'fr', 'es'] as const;

export function LocaleSwitcher() {
  const { i18n } = useTranslation();
  return (
    <select
      aria-label="Language"
      value={i18n.language}
      onChange={(e) => void i18n.changeLanguage(e.target.value)}
      className="h-11 rounded-lg border border-border-subtle bg-surface px-2 text-text-1"
    >
      {LOCALES.map((l) => (
        <option key={l} value={l}>
          {l.toUpperCase()}
        </option>
      ))}
    </select>
  );
}
