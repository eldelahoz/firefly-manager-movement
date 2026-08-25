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
  | 'failed';

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
