import { useTranslation } from 'react-i18next';

export type PillStatus =
  | 'imported'
  | 'matched'
  | 'missing'
  | 'created'
  | 'ignored'
  | 'duplicate'
  | 'extra'
  | 'already_created'
  | 'failed'
  | 'parsed'
  | 'reconciled'
  | 'applied';

const STYLES: Record<PillStatus, string> = {
  imported: 'bg-border-subtle text-text-2',
  matched: 'bg-status-matched-bg text-status-matched-txt',
  missing: 'bg-status-missing-bg text-status-missing-txt',
  extra: 'bg-status-extra-bg text-status-extra-txt',
  created: 'bg-accent-soft text-accent',
  ignored: 'bg-border-subtle text-text-3',
  duplicate: 'bg-status-extra-bg text-status-extra-txt',
  already_created: 'bg-border-subtle text-text-2',
  failed: 'bg-status-extra-bg text-status-extra-txt',
  parsed: 'bg-border-subtle text-text-2',
  reconciled: 'bg-status-matched-bg text-status-matched-txt',
  applied: 'bg-accent-soft text-accent',
};

export function StatusPill({ status }: { status: PillStatus }) {
  const { t } = useTranslation();
  return (
    <span
      className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-[11px] font-semibold uppercase tracking-wide ${STYLES[status]}`}
    >
      {t(`status.${status}`)}
    </span>
  );
}
