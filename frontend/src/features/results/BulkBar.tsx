import { useTranslation } from 'react-i18next';
import { Pencil, X } from 'lucide-react';
import { Button } from '@/shared/ui/button';

/** Floating actions for the current selection; mounted only while something is selected. */
export function BulkBar({
  count,
  onPersonalize,
  onCreate,
  onClear,
}: {
  count: number;
  onPersonalize: () => void;
  onCreate: () => void;
  onClear: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div
      role="toolbar"
      aria-label={t('results.selection')}
      className="ffm-bulkbar fixed inset-x-3 bottom-[4.5rem] z-30 flex flex-wrap items-center justify-between gap-2 rounded-xl bg-text-1 py-2 pr-2 pl-4 text-surface shadow-lg [view-transition-name:bulk-bar] md:inset-x-auto md:bottom-5 md:left-[calc(50%+var(--sidebar-w,0px)/2)] md:-translate-x-1/2 md:flex-nowrap"
    >
      <span className="min-h-9 content-center font-semibold whitespace-nowrap tabular-nums">
        {t('results.selected', { count })}
      </span>
      <div className="grid w-full grid-cols-2 gap-2 md:flex md:w-auto">
        <Button
          variant="ghost"
          className="text-surface hover:bg-surface/15 hover:text-surface"
          onClick={onPersonalize}
        >
          <Pencil className="size-4" aria-hidden="true" />
          {t('results.personalize')} ({count})
        </Button>
        <Button onClick={onCreate}>
          {t('results.apply')} ({count})
        </Button>
      </div>
      <Button
        variant="ghost"
        size="icon"
        aria-label={t('results.clearSelection')}
        title={t('results.clearSelection')}
        className="text-surface hover:bg-surface/15 hover:text-surface absolute top-1.5 right-2 md:static"
        onClick={onClear}
      >
        <X className="size-4" />
      </Button>
    </div>
  );
}
