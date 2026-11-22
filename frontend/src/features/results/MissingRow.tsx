import { useTranslation } from 'react-i18next';
import { EyeOff, Pencil } from 'lucide-react';
import type { MovementDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Checkbox } from '@/shared/ui/checkbox';
import { StatusPill } from '@/shared/ui/StatusPill';
import { CELL, GRID_COLS, GROUP_ACCENT } from './tableLayout';

export function MissingRow({
  movement,
  currency,
  selected,
  isLast = false,
  onToggle,
  report,
  onEdit,
  onIgnore,
}: {
  movement: MovementDTO;
  currency: string;
  selected?: boolean;
  isLast?: boolean;
  onToggle?: (id: number) => void;
  report?: { outcome: 'created' | 'already_created' | 'failed'; reason?: string };
  onEdit?: () => void;
  onIgnore?: (id: number) => void;
}) {
  const { t, i18n } = useTranslation();
  return (
    <div
      role="row"
      data-vt-key={`mv-${movement.id}`}
      className={`grid ${GRID_COLS.missing} items-center gap-3 px-4 py-3 text-sm transition-colors ${
        isLast ? '' : 'border-b border-border-subtle'
      } ${GROUP_ACCENT.missing.rowHover}`}
    >
      <div role="cell" className={`flex justify-center ${CELL.missing.select}`}>
        <Checkbox
          aria-label={movement.description}
          checked={selected ?? false}
          onCheckedChange={() => onToggle?.(movement.id)}
        />
      </div>
      <span role="cell" className={CELL.missing.date}>
        {formatDate(movement.date, i18n.language)}
      </span>
      <span role="cell" className={`min-w-0 ${CELL.missing.desc}`} title={movement.description}>
        <span className="block truncate">{movement.description}</span>
        {report?.outcome === 'failed' && report.reason ? (
          <span className="block text-xs text-danger">{report.reason}</span>
        ) : null}
      </span>
      <div role="cell" className={`text-right ${CELL.missing.amount}`}>
        <MoneyText cents={movement.amount_cents} currency={currency} />
      </div>
      {report ? (
        <div
          role="cell"
          title={report.reason ?? ''}
          className={`flex justify-center ${CELL.missing.statusReport}`}
        >
          <StatusPill status={report.outcome} />
        </div>
      ) : (
        <div role="cell" className={`flex justify-center ${CELL.missing.status}`}>
          <StatusPill status={movement.status} />
        </div>
      )}
      <span role="cell" className={`flex items-center justify-end gap-1 ${CELL.missing.actions}`}>
        <Button
          variant="ghost"
          size="icon-sm"
          className="@max-[40rem]:size-10"
          aria-label={t('results.edit')}
          title={t('results.edit')}
          onClick={() => onEdit?.()}
        >
          <Pencil className="size-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          className="@max-[40rem]:size-10"
          aria-label={t('results.ignore')}
          title={t('results.ignore')}
          onClick={() => onIgnore?.(movement.id)}
        >
          <EyeOff className="size-4" />
        </Button>
      </span>
    </div>
  );
}
