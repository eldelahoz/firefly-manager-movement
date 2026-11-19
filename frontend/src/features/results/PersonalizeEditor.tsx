import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Paperclip, X } from 'lucide-react';
import type { Ff3Account } from '@/shared/api/dto';
import type { Customization } from './customization';
import { TagsPicker } from './TagsPicker';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';

/** RF-08 fields. `single` edits one movement (description prefilled with the statement text);
 *  `bulk` returns only the fields the user filled (fill-only), so empty fields leave rows untouched. */
export function PersonalizeEditor({
  mode,
  initial,
  statementDescription = '',
  defaultTag,
  direction,
  accounts,
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
  direction?: 'withdrawal' | 'deposit';
  accounts: Ff3Account[];
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
  const [counterpartyId, setCounterpartyId] = useState<string>(
    initial.counterpartyId ? String(initial.counterpartyId) : 'none',
  );
  const [description, setDescription] = useState(
    initial.description ?? (bulk ? '' : statementDescription),
  );
  const [category, setCategory] = useState(initial.category ?? 'none');
  const [budget, setBudget] = useState(initial.budget ?? 'none');
  const [notes, setNotes] = useState(initial.notes ?? '');
  const [selectedTags, setSelectedTags] = useState<string[]>(
    initial.tags ?? (defaultTag ? [defaultTag] : []),
  );
  const [tagsTouched, setTagsTouched] = useState(false);
  const [attachment, setAttachment] = useState(initial.attachment);
  const fileInput = useRef<HTMLInputElement>(null);

  // Synchronous, so an Escape right after typing already sees the form as dirty.
  const touch = () => onDirtyChange?.(true);

  const filteredAccounts =
    direction === 'withdrawal'
      ? accounts.filter((a) => a.type === 'expense')
      : direction === 'deposit'
        ? accounts.filter((a) => a.type === 'revenue')
        : [];

  function handleSave() {
    onSave({
      counterpartyId: counterpartyId === 'none' ? undefined : Number(counterpartyId),
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
        {bulk && <p className="text-xs text-text-3">{t('results.bulkHint')}</p>}
        {select(
          t('results.counterparty'),
          counterpartyId,
          setCounterpartyId,
          filteredAccounts.map((a) => ({
            value: String(a.id),
            label: `${a.name} (${a.currency})`,
          })),
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
              setSelectedTags(v);
              setTagsTouched(true);
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
