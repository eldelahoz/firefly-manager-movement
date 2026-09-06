import { useTranslation } from 'react-i18next';
import { EyeOff, Pencil } from 'lucide-react';
import type { MovementDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Checkbox } from '@/shared/ui/checkbox';
import { StatusPill } from '@/shared/ui/StatusPill';
import { GRID_COLS, GROUP_ACCENT } from './tableLayout';

export function MissingRow({
  movement,
  currency,
  selected,
  onToggle,
  report,
  onEdit,
  onIgnore,
}: {
  movement: MovementDTO;
  currency: string;
  selected?: boolean;
  onToggle?: (id: number) => void;
  report?: { outcome: 'created' | 'already_created' | 'failed'; reason?: string };
  onEdit?: () => void;
  onIgnore?: (id: number) => void;
}) {
  const { t, i18n } = useTranslation();
  return (
    <div
      className={`border-b border-border-subtle py-3 text-sm transition-colors ${GROUP_ACCENT.missing.rowHover}`}
    >
      <div role="row" className={`grid ${GRID_COLS.missing} items-center gap-3 px-2`}>
        <div role="cell">
          <Checkbox
            aria-label={movement.description}
            checked={selected ?? false}
            onCheckedChange={() => onToggle?.(movement.id)}
          />
        </div>
        <span role="cell">{formatDate(movement.date, i18n.language)}</span>
        <span role="cell" className="min-w-0 truncate" title={movement.description}>
          {movement.description}
        </span>
        <div role="cell">
          <MoneyText cents={movement.amount_cents} currency={currency} className="text-right" />
        </div>
        {report ? (
          <div role="cell" title={report.reason ?? ''} className="flex justify-center">
            <StatusPill status={report.outcome} />
          </div>
        ) : (
          <div role="cell" className="flex justify-center">
            <StatusPill status={movement.status} />
          </div>
        )}
        <span role="cell" className="flex items-center justify-center gap-1">
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={t('results.edit')}
            title={t('results.edit')}
            onClick={() => onEdit?.()}
          >
            <Pencil className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            aria-label={t('results.ignore')}
            title={t('results.ignore')}
            onClick={() => onIgnore?.(movement.id)}
          >
            <EyeOff className="size-4" />
          </Button>
        </span>
      </div>
    </div>
  );
}
