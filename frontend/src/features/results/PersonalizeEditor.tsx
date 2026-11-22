import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Paperclip, X } from 'lucide-react';
import type { Ff3Account } from '@/shared/api/dto';
import type { Counterparty, Customization, Direction } from './customization';
import { transactionType } from './customization';
import { TagsPicker } from './TagsPicker';
import { TransactionTypeIcon } from './TransactionTypeIcon';
import { Button } from '@/shared/ui/button';
import { Combobox, type ComboboxOption } from '@/shared/ui/combobox';
import { Input } from '@/shared/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';

// Picker values: '' = bulk, leave unchanged; 'cash'; 'id:<ff3 id>'; 'new:<name>'.
function encode(c: Counterparty | undefined, bulk: boolean): string {
  if (c === undefined) return bulk ? '' : 'cash';
  if (c === null) return 'cash';
  return 'id' in c ? `id:${c.id}` : `new:${c.name}`;
}
function decode(v: string): Counterparty | undefined {
  if (v === '') return undefined;
  if (v === 'cash') return null;
  return v.startsWith('id:') ? { id: Number(v.slice(3)) } : { name: v.slice(4) };
}

/** RF-08 fields. `single` edits one movement (description prefilled with the statement text);
 *  `bulk` returns only the fields the user filled (fill-only), so empty fields leave rows untouched. */
export function PersonalizeEditor({
  mode,
  initial,
  statementDescription = '',
  defaultTag,
  direction,
  accounts,
  batchAccountId,
  currency,
  tags,
  categories,
  budgets,
  onSave,
  onCancel,
  onDirtyChange,
  submitLabel,
}: {
  mode: 'single' | 'bulk';
  initial: Customization;
  statementDescription?: string;
  defaultTag: string | null;
  /** RF-08: counterparties listed by direction; undefined (bulk with mixed directions) lists none. */
  direction?: Direction;
  accounts: Ff3Account[];
  /** The batch's own asset account (A): never offered as a counterparty. */
  batchAccountId: number | null;
  /** Own accounts in another currency would need a foreign amount (out of scope, rejected by the backend). */
  currency: string;
  tags: string[];
  categories: string[];
  budgets: string[];
  onSave: (c: Customization) => void;
  onCancel: () => void;
  onDirtyChange?: (dirty: boolean) => void;
  submitLabel: string;
}) {
  const { t } = useTranslation();
  const bulk = mode === 'bulk';
  const [counterparty, setCounterparty] = useState(() => encode(initial.counterparty, bulk));
  const [description, setDescription] = useState(
    initial.description ?? (bulk ? '' : statementDescription),
  );
  const [category, setCategory] = useState(initial.category ?? 'none');
  const [budget, setBudget] = useState(initial.budget ?? 'none');
  const [notes, setNotes] = useState(initial.notes ?? '');
  // Derived until the user picks, so a default tag from settings that loads late still applies.
  const [tagsDraft, setTagsDraft] = useState<string[] | null>(null);
  const tagsTouched = tagsDraft !== null;
  const selectedTags = tagsDraft ?? initial.tags ?? (defaultTag ? [defaultTag] : []);
  const [attachment, setAttachment] = useState(initial.attachment);
  const fileInput = useRef<HTMLInputElement>(null);

  // Synchronous, so an Escape right after typing already sees the form as dirty.
  const touch = () => onDirtyChange?.(true);

  const out = direction === 'withdrawal';
  const accountLabel = (a: Ff3Account) => `${a.name} (${a.currency})`;
  const badge = (a: Ff3Account) =>
    a.role === 'ccAsset'
      ? t('results.cp.creditCard')
      : a.liability_type
        ? t(`results.cp.${a.liability_type}`, { defaultValue: a.liability_type })
        : undefined;
  const groupAccounts = [
    [
      'results.cp.' + (out ? 'expense' : 'revenue'),
      accounts.filter((a) => a.type === (out ? 'expense' : 'revenue')),
    ],
    [
      'results.cp.liabilities',
      // The backend passes Firefly III's own "liabilities" through; accept the singular too.
      accounts.filter((a) => a.type === 'liabilities' || a.type === 'liability'),
    ],
    [
      'results.cp.own',
      accounts.filter(
        (a) => a.type === 'asset' && a.id !== batchAccountId && a.currency === currency,
      ),
    ],
  ] as const;
  const groups = groupAccounts.map(([key, list]) => ({
    label: t(key),
    options: list.map((a) => ({ value: `id:${a.id}`, label: accountLabel(a), badge: badge(a) })),
  }));
  const footer = (query: string): ComboboxOption[] => {
    const taken = groupAccounts.some(([, list]) =>
      list.some((a) => a.name.toLowerCase() === query.toLowerCase()),
    );
    return [
      ...(query && !taken
        ? [
            {
              value: `new:${query}`,
              label: t(out ? 'results.cp.createExpense' : 'results.cp.createRevenue', {
                name: query,
              }),
            },
          ]
        : []),
      { value: 'cash', label: t('results.cp.cashOption') },
      // Bulk: a way back after picking by mistake ('' leaves each row's own choice).
      ...(bulk ? [{ value: '', label: t('results.cp.keep') }] : []),
    ];
  };
  const chosen = decode(counterparty);
  const chosenType =
    direction && chosen !== undefined ? transactionType(direction, chosen, accounts) : undefined;
  const chosenAccount =
    chosen && 'id' in chosen ? accounts.find((a) => a.id === chosen.id) : undefined;
  const displayValue =
    chosen === undefined
      ? ''
      : chosen === null
        ? t('results.cp.cashOption')
        : 'id' in chosen
          ? chosenAccount
            ? accountLabel(chosenAccount)
            : `#${chosen.id}`
          : t(out ? 'results.cp.newExpense' : 'results.cp.newRevenue', { name: chosen.name });

  function handleSave() {
    onSave({
      counterparty: chosen,
      // The prefilled statement text is the default: store a description only when it differs.
      description: description && description !== statementDescription ? description : undefined,
      category: category === 'none' ? undefined : category,
      budget: budget === 'none' ? undefined : budget,
      tags: bulk && !tagsTouched ? undefined : selectedTags,
      notes: notes || undefined,
      attachment: bulk ? undefined : attachment,
    });
  }

  const select = (
    label: string,
    value: string,
    onChange: (v: string) => void,
    options: { value: string; label: string }[],
  ) => (
    <div>
      <p className="mb-1 text-sm text-text-2">{label}</p>
      <Select
        value={value}
        onValueChange={(v) => {
          onChange(v);
          touch();
        }}
      >
        <SelectTrigger className="w-full" aria-label={label}>
          <SelectValue placeholder="—" />
        </SelectTrigger>
        <SelectContent position="popper">
          <SelectItem value="none">—</SelectItem>
          {options.map((o) => (
            <SelectItem key={o.value} value={o.value}>
              {o.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="min-h-0 flex-1 space-y-4 overflow-y-auto px-4 pb-4">
        {direction ? (
          <Combobox
            label={t(out ? 'results.cp.destination' : 'results.cp.source')}
            value={counterparty}
            displayValue={displayValue}
            placeholder={bulk ? t('results.cp.keep') : undefined}
            groups={groups}
            footer={footer}
            onChange={(v) => {
              setCounterparty(v);
              touch();
            }}
            description={
              chosenType && (
                <span className="inline-flex items-center gap-1">
                  <TransactionTypeIcon type={chosenType} />
                  {t(`results.cp.willCreate.${chosenType}`)}
                </span>
              )
            }
          />
        ) : (
          <p className="text-sm text-text-2">{t('results.cp.mixed')}</p>
        )}
        <label className="block text-sm text-text-2">
          {t('results.descriptionField')}
          <Input
            value={description}
            onChange={(e) => {
              setDescription(e.target.value);
              touch();
            }}
            className="mt-1"
          />
        </label>
        <div>
          <p className="mb-1 text-sm text-text-2">{t('results.tags')}</p>
          <TagsPicker
            value={selectedTags}
            options={tags}
            onChange={(v) => {
              setTagsDraft(v);
              touch();
            }}
          />
        </div>
        {select(
          t('results.category'),
          category,
          setCategory,
          categories.map((c) => ({ value: c, label: c })),
        )}
        {select(
          t('results.budget'),
          budget,
          setBudget,
          budgets.map((b) => ({ value: b, label: b })),
        )}
        <label className="block text-sm text-text-2">
          {t('results.notes')}
          <Input
            value={notes}
            onChange={(e) => {
              setNotes(e.target.value);
              touch();
            }}
            className="mt-1"
          />
        </label>
        {!bulk && (
          <div>
            <p className="mb-1 text-sm text-text-2">{t('results.attachment')}</p>
            <input
              ref={fileInput}
              type="file"
              aria-label={t('results.attachment')}
              className="sr-only"
              tabIndex={-1}
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (file) setAttachment({ filename: file.name, file });
                e.target.value = '';
                touch();
              }}
            />
            {attachment ? (
              <span className="inline-flex max-w-full items-center gap-1 rounded-full bg-border-subtle py-0.5 pr-1 pl-3 text-sm text-text-1">
                <Paperclip className="size-3.5 shrink-0" aria-hidden="true" />
                <span className="min-w-0 truncate">{attachment.filename}</span>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label={t('results.removeAttachment')}
                  title={t('results.removeAttachment')}
                  onClick={() => {
                    setAttachment(undefined);
                    touch();
                  }}
                >
                  <X className="size-3.5" />
                </Button>
              </span>
            ) : (
              <Button variant="outline" onClick={() => fileInput.current?.click()}>
                <Paperclip className="size-4" aria-hidden="true" />
                {t('results.attachFile')}
              </Button>
            )}
          </div>
        )}
      </div>
      <div className="flex justify-end gap-2 border-t border-border-subtle p-4">
        <Button variant="cancel" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
        <Button onClick={handleSave}>{submitLabel}</Button>
      </div>
    </div>
  );
}
