import { useTranslation } from 'react-i18next';
import type { Ff3Account, MovementDTO } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Checkbox } from '@/shared/ui/checkbox';
import { StatusPill } from '@/shared/ui/StatusPill';
import type { Customization } from './customization';
import { PersonalizeEditor } from './PersonalizeEditor';

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
    <div className="border-b border-border-subtle py-3 text-sm">
      <div className="grid grid-cols-[2rem_7rem_1fr_8rem_6rem_11rem] items-center gap-2">
        <Checkbox
          aria-label={movement.description}
          checked={selected ?? false}
          onCheckedChange={() => onToggle?.(movement.id)}
        />
        <span>{formatDate(movement.date, i18n.language)}</span>
        <span className="min-w-0 truncate" title={movement.description}>
          {movement.description}
        </span>
        <MoneyText cents={movement.amount_cents} currency={currency} />
        {report ? (
          <span title={report.reason ?? ''}>
            <StatusPill status={report.outcome} />
          </span>
        ) : (
          <StatusPill status={movement.status} />
        )}
        <span className="flex gap-1">
          <Button variant="ghost" size="sm" onClick={() => onEdit?.()} disabled={editing}>
            {t('results.edit')}
          </Button>
          <Button variant="ghost" size="sm" onClick={() => onIgnore?.(movement.id)}>
            {t('results.ignore')}
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
