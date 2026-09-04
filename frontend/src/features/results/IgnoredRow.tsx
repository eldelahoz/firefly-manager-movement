import { useTranslation } from 'react-i18next';
import type { MovementDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { StatusPill } from '@/shared/ui/StatusPill';
import { GRID_COLS } from './tableLayout';

export function IgnoredRow({
  movement,
  currency,
  onUnignore,
}: {
  movement: MovementDTO;
  currency: string;
  onUnignore?: () => void;
}) {
  const { t, i18n } = useTranslation();
  return (
    <div
      className={`grid ${GRID_COLS.ignored} items-center gap-2 border-b border-border-subtle py-3 text-sm`}
    >
      <span>{formatDate(movement.date, i18n.language)}</span>
      <span className="min-w-0 truncate" title={movement.description}>
        {movement.description}
      </span>
      <MoneyText cents={movement.amount_cents} currency={currency} />
      <StatusPill status="ignored" />
      <span>
        <Button variant="ghost" size="sm" onClick={onUnignore}>
          {t('results.unignore')}
        </Button>
      </span>
    </div>
  );
}
