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
  imported: 'bg-canvas text-text-2 border border-border-subtle',
  matched: 'bg-status-matched-bg text-status-matched-txt',
  missing: 'bg-status-missing-bg text-status-missing-txt',
  extra: 'bg-status-extra-bg text-status-extra-txt',
  created: 'bg-accent-soft text-accent',
  ignored: 'bg-canvas text-text-3 border border-border-subtle',
  duplicate: 'bg-surface text-status-extra-txt border border-status-extra',
  already_created: 'bg-canvas text-text-2 border border-border-subtle',
  failed: 'bg-status-extra-bg text-status-extra-txt',
};

export function StatusPill({ status }: { status: PillStatus }) {
  const { t } = useTranslation();
  return (
    <span
      className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${STYLES[status]}`}
    >
      {t(`status.${status}`)}
    </span>
  );
}
