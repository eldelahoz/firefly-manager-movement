import { useTranslation } from 'react-i18next';

/** Per-group CSS grid templates. Header and rows MUST use the same template. */
export const GRID_COLS = {
  matched: 'grid-cols-[7rem_1fr_1fr_8rem_7rem]',
  missing: 'grid-cols-[2rem_7rem_1fr_8rem_7rem_11rem]',
  extra: 'grid-cols-[7rem_1fr_8rem_7rem_9rem]',
  ignored: 'grid-cols-[7rem_1fr_8rem_7rem_7rem]',
} as const;

export type GroupKey = keyof typeof GRID_COLS;

const LABELS: Record<GroupKey, string[]> = {
  matched: [
    'col.date',
    'col.descriptionStatement',
    'col.descriptionFf3',
    'col.amount',
    'col.status',
  ],
  missing: [
    'col.selection',
    'col.date',
    'col.description',
    'col.amount',
    'col.status',
    'col.actions',
  ],
  extra: ['col.date', 'col.description', 'col.amount', 'col.status', 'col.actions'],
  ignored: ['col.date', 'col.description', 'col.amount', 'col.status', 'col.actions'],
};

export function TableHeader({ group }: { group: GroupKey }) {
  const { t } = useTranslation();
  return (
    <div
      className={`grid ${GRID_COLS[group]} gap-2 border-b border-border px-2 py-1.5 text-xs font-semibold uppercase tracking-wide text-text-3`}
    >
      {LABELS[group].map((k) => (
        <span key={k} className={k === 'col.actions' ? 'text-center' : undefined}>
          {t(`results.${k}`)}
        </span>
      ))}
    </div>
  );
}
