import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

export type PillStatus = string;

const PILL_BASE =
  'inline-flex w-fit max-w-full items-center overflow-hidden rounded-full px-2.5 py-0.5 text-[11px] font-semibold uppercase tracking-wide';

const NEUTRAL = 'bg-border-subtle text-text-2';

const STYLES: Record<string, string> = {
  imported: 'bg-border-subtle text-text-2',
  matched: 'bg-status-matched-bg text-status-matched-txt',
  missing: 'bg-status-missing-bg text-status-missing-txt',
  extra: 'bg-status-extra-bg text-status-extra-txt',
  created: 'bg-accent-soft text-accent',
  ignored: 'bg-border-subtle text-text-3',
  duplicate: 'bg-status-extra-bg text-status-extra-txt',
  already_created: 'bg-border-subtle text-text-2',
  failed: 'bg-status-extra-bg text-status-extra-txt',
  parsed: 'bg-status-missing-bg text-status-missing-txt',
  reconciled: 'bg-status-matched-bg text-status-matched-txt',
  applied: 'bg-accent-soft text-accent',
};

/** Generic text pill. Defaults to a neutral gray; pass `className` to override the color. */
export function Pill({
  children,
  className = NEUTRAL,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <span className={`${PILL_BASE} ${className}`}>{children}</span>;
}

export function StatusPill({ status }: { status: string }) {
  const { t } = useTranslation();
  return (
    <Pill className={STYLES[status] ?? NEUTRAL}>
      <span className="truncate">{t(`status.${status}`)}</span>
    </Pill>
  );
}
