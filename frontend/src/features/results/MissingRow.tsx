import { useTranslation } from 'react-i18next';
import { EyeOff, Pencil } from 'lucide-react';
import type { Ff3Account, MovementDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Checkbox } from '@/shared/ui/checkbox';
import { StatusPill } from '@/shared/ui/StatusPill';
import type { Customization } from './customization';
import { PersonalizeEditor } from './PersonalizeEditor';
import { GRID_COLS, GROUP_ACCENT } from './tableLayout';

export function MissingRow({
  movement,
  currency,
  selected,
  onToggle,
  report,
  editing,
  onEdit,
  onIgnore,
  accounts,
  tags,
  categories,
  budgets,
  defaultTag,
  onSave,
  onCancel,
}: {
  movement: MovementDTO;
  currency: string;
  selected?: boolean;
  onToggle?: (id: number) => void;
  report?: { outcome: 'created' | 'already_created' | 'failed'; reason?: string };
  editing?: boolean;
  onEdit?: () => void;
  onIgnore?: (id: number) => void;
  accounts: Ff3Account[];
  tags: string[];
  categories: string[];
  budgets: string[];
  defaultTag: string | null;
  onSave: (c: Customization) => void;
  onCancel: () => void;
}) {
  const { t, i18n } = useTranslation();
  const direction: 'withdrawal' | 'deposit' = movement.amount_cents < 0 ? 'withdrawal' : 'deposit';
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
            disabled={editing}
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
      {editing && (
        <PersonalizeEditor
          initial={{}}
          defaultTag={defaultTag}
          direction={direction}
          accounts={accounts}
          tags={tags}
          categories={categories}
          budgets={budgets}
          onSave={onSave}
          onCancel={onCancel}
          submitLabel={t('results.confirm')}
        />
      )}
    </div>
  );
}
