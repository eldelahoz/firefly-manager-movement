import { useTranslation } from 'react-i18next';

/** Per-group CSS grid templates. Header and rows MUST use the same template.
 *  Shared columns (date/amount/status/actions) keep identical widths across groups. */
export const GRID_COLS = {
  matched: 'grid-cols-[7.5rem_1fr_1fr_8rem_7.5rem]',
  missing: 'grid-cols-[2rem_7.5rem_1fr_8rem_7.5rem_6rem]',
  extra: 'grid-cols-[7.5rem_1fr_8rem_7.5rem_6rem]',
  ignored: 'grid-cols-[7.5rem_1fr_8rem_7.5rem_6rem]',
} as const;

export type GroupKey = keyof typeof GRID_COLS;

/** Per-group visual identity: header underline, label tint, row hover tint. */
export const GROUP_ACCENT = {
  matched: {
    underline: 'border-b-status-matched',
    text: 'text-status-matched-txt',
    rowHover: 'hover:bg-status-matched-bg/60',
  },
  missing: {
    underline: 'border-b-status-missing',
    text: 'text-status-missing-txt',
    rowHover: 'hover:bg-status-missing-bg/60',
  },
  extra: {
    underline: 'border-b-status-extra',
    text: 'text-status-extra-txt',
    rowHover: 'hover:bg-status-extra-bg/60',
  },
  ignored: {
    underline: 'border-b-border-strong',
    text: 'text-text-3',
    rowHover: 'hover:bg-canvas',
  },
} as const;

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
  const accent = GROUP_ACCENT[group];
  return (
    <div
      role="row"
      className={`grid ${GRID_COLS[group]} gap-3 border-b-2 bg-table-header px-2 py-2.5 text-xs font-semibold uppercase tracking-wide ${accent.underline} ${accent.text}`}
    >
      {LABELS[group].map((k) => (
        <span
          key={k}
          role="columnheader"
          className={
            k === 'col.actions' || k === 'col.status'
              ? 'text-center'
              : k === 'col.amount'
                ? 'text-right'
                : undefined
          }
        >
          {t(`results.${k}`)}
        </span>
      ))}
    </div>
  );
}
