import { useTranslation } from 'react-i18next';

/** Per-group CSS grid templates. Header and rows MUST use the same template.
 *  Shared columns (date/amount/status/actions) keep identical widths across groups. */
export const GRID_COLS = {
  matched:
    'grid-cols-[7.5rem_minmax(0,1fr)_minmax(0,1fr)_8rem_7.5rem] @max-[40rem]:grid-cols-[minmax(0,1fr)_auto] @max-[40rem]:gap-y-1',
  missing:
    'grid-cols-[2rem_7.5rem_minmax(0,1fr)_8rem_7.5rem_6rem] @max-[40rem]:grid-cols-[2rem_minmax(0,1fr)_auto] @max-[40rem]:gap-y-1',
  extra:
    'grid-cols-[7.5rem_minmax(0,1fr)_8rem_7.5rem_6rem] @max-[40rem]:grid-cols-[minmax(0,1fr)_auto] @max-[40rem]:gap-y-1',
  ignored:
    'grid-cols-[7.5rem_minmax(0,1fr)_8rem_7.5rem_6rem] @max-[40rem]:grid-cols-[minmax(0,1fr)_auto] @max-[40rem]:gap-y-1',
} as const;

export type GroupKey = keyof typeof GRID_COLS;

/** Stacked placement below a 40rem container (ancestor needs the `@container` class, see TABLE_WRAPPER):
 *  line 1 = description + amount, line 2 = date + actions, status only where it carries information.
 *  Class strings stay literal so Tailwind can detect them. */
const SM_HIDE = '@max-[40rem]:hidden';
const SM_META = '@max-[40rem]:text-xs @max-[40rem]:text-text-2';
export const CELL = {
  matched: {
    date: `@max-[40rem]:col-start-1 @max-[40rem]:row-start-2 ${SM_META}`,
    desc: '@max-[40rem]:col-start-1 @max-[40rem]:row-start-1',
    ff3: `@max-[40rem]:col-span-2 @max-[40rem]:col-start-1 @max-[40rem]:row-start-3 ${SM_META}`,
    amount: '@max-[40rem]:col-start-2 @max-[40rem]:row-start-1',
    status: SM_HIDE,
  },
  missing: {
    select:
      '@max-[40rem]:col-start-1 @max-[40rem]:row-span-2 @max-[40rem]:row-start-1 @max-[40rem]:self-start',
    date: `@max-[40rem]:col-start-2 @max-[40rem]:row-start-2 ${SM_META}`,
    desc: '@max-[40rem]:col-start-2 @max-[40rem]:row-start-1',
    amount: '@max-[40rem]:col-start-3 @max-[40rem]:row-start-1',
    // An apply outcome (failed / already created) must stay visible on phones; the plain "missing" pill is redundant.
    statusReport:
      '@max-[40rem]:col-span-2 @max-[40rem]:col-start-2 @max-[40rem]:row-start-3 @max-[40rem]:justify-start',
    status: SM_HIDE,
    actions: '@max-[40rem]:col-start-3 @max-[40rem]:row-start-2',
  },
  extra: {
    date: `@max-[40rem]:col-start-1 @max-[40rem]:row-start-2 ${SM_META}`,
    desc: '@max-[40rem]:col-start-1 @max-[40rem]:row-start-1',
    amount: '@max-[40rem]:col-start-2 @max-[40rem]:row-start-1',
    status:
      '@max-[40rem]:col-span-2 @max-[40rem]:col-start-1 @max-[40rem]:row-start-3 @max-[40rem]:justify-start',
    actions: '@max-[40rem]:col-start-2 @max-[40rem]:row-start-2',
  },
  ignored: {
    date: `@max-[40rem]:col-start-1 @max-[40rem]:row-start-2 ${SM_META}`,
    desc: '@max-[40rem]:col-start-1 @max-[40rem]:row-start-1',
    amount: '@max-[40rem]:col-start-2 @max-[40rem]:row-start-1',
    status: SM_HIDE,
    actions: '@max-[40rem]:col-start-2 @max-[40rem]:row-start-2',
  },
} as const;

/** Shared wrapper for every results table: also the size container the stacked layout queries. */
export const TABLE_WRAPPER =
  '@container overflow-hidden rounded-lg border border-border-subtle bg-surface shadow-sm';

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
      className={`grid ${GRID_COLS[group]} @max-[40rem]:sr-only gap-3 border-b-2 bg-table-header px-4 py-2.5 text-xs font-semibold uppercase tracking-wide ${accent.underline} ${accent.text}`}
    >
      {LABELS[group].map((k) => (
        <span
          key={k}
          role="columnheader"
          className={
            k === 'col.actions'
              ? 'text-right'
              : k === 'col.status'
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
