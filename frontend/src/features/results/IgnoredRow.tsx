import { useTranslation } from 'react-i18next';
import { Eye } from 'lucide-react';
import type { MovementDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { StatusPill } from '@/shared/ui/StatusPill';
import { GRID_COLS, GROUP_ACCENT } from './tableLayout';

export function IgnoredRow({
  movement,
  currency,
  isLast = false,
  onUnignore,
}: {
  movement: MovementDTO;
  currency: string;
  isLast?: boolean;
  onUnignore?: () => void;
}) {
  const { t, i18n } = useTranslation();
  return (
    <div
      role="row"
      className={`grid ${GRID_COLS.ignored} items-center gap-3 px-2 py-3 text-sm transition-colors ${
        isLast ? '' : 'border-b border-border-subtle'
      } ${GROUP_ACCENT.ignored.rowHover}`}
    >
      <span role="cell">{formatDate(movement.date, i18n.language)}</span>
      <span role="cell" className="min-w-0 truncate" title={movement.description}>
        {movement.description}
      </span>
      <div role="cell" className="text-right">
        <MoneyText cents={movement.amount_cents} currency={currency} />
      </div>
      <div role="cell" className="flex justify-center">
        <StatusPill status="ignored" />
      </div>
      <span role="cell" className="flex items-center justify-center">
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={t('results.unignore')}
          title={t('results.unignore')}
          onClick={onUnignore}
        >
          <Eye className="size-4" />
        </Button>
      </span>
    </div>
  );
}
