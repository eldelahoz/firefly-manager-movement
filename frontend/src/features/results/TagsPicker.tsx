import { useTranslation } from 'react-i18next';
import { cn } from '@/shared/lib/utils';

export function TagsPicker({
  value,
  options,
  onChange,
}: {
  value: string[];
  options: string[];
  onChange: (tags: string[]) => void;
}) {
  const { t } = useTranslation();
  const all = [...new Set([...options, ...value])];
  function toggle(tag: string) {
    onChange(value.includes(tag) ? value.filter((t) => t !== tag) : [...value, tag]);
  }
  if (all.length === 0) {
    return <p className="text-xs text-text-3">{t('results.noTags')}</p>;
  }
  return (
    <div className="flex flex-wrap gap-1">
      {all.map((tag) => {
        const active = value.includes(tag);
        return (
          <button
            key={tag}
            type="button"
            aria-pressed={active}
            onClick={() => toggle(tag)}
            className={cn(
              'rounded-full border px-2 py-0.5 text-xs transition-colors',
              active
                ? 'border-accent bg-accent text-accent-foreground'
                : 'border-border-subtle bg-surface text-text-2 hover:bg-accent-soft',
            )}
          >
            {tag}
          </button>
        );
      })}
    </div>
  );
}
