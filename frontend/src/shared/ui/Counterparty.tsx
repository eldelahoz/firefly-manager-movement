import { useTranslation } from 'react-i18next';
import { Pill } from './StatusPill';

const TYPE_KEY: Record<string, string> = {
  'Expense account': 'expense',
  'Revenue account': 'revenue',
  'Asset account': 'asset',
  'Cash account': 'cash',
  Loan: 'loan',
  Debt: 'debt',
  Mortgage: 'mortgage',
  'Initial balance account': 'initialBalance',
  'Reconciliation account': 'reconciliation',
  'Liability credit account': 'liabilityCredit',
};

/** RF-14 v0.2 / RF-07 v0.11: the opposite account of a Firefly III transaction, name + type badge.
 *  Renders nothing without data (old snapshots). The Cash account shows only its badge. */
export function Counterparty({ name, type }: { name?: string | null; type?: string | null }) {
  const { t } = useTranslation();
  if (!name && !type) return null;
  const key = type ? TYPE_KEY[type] : undefined;
  return (
    <span className="flex min-w-0 items-center gap-1.5">
      {name && key !== 'cash' && (
        <span className="min-w-0 truncate" title={name}>
          {name}
        </span>
      )}
      {type && (
        <span className="shrink-0 whitespace-nowrap">
          <Pill>{key ? t(`results.cp.badge.${key}`) : type}</Pill>
        </span>
      )}
    </span>
  );
}
