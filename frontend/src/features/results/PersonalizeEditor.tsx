import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Ff3Account } from '@/shared/api/dto';
import type { Customization } from './customization';
import { TagsPicker } from './TagsPicker';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';

export function PersonalizeEditor({
  initial,
  defaultTag,
  direction,
  accounts,
  tags,
  categories,
  budgets,
  onSave,
  onCancel,
  submitLabel,
}: {
  initial: Customization;
  defaultTag: string | null;
  direction: 'withdrawal' | 'deposit';
  accounts: Ff3Account[];
  tags: string[];
  categories: string[];
  budgets: string[];
  onSave: (c: Customization) => void;
  onCancel: () => void;
  submitLabel: string;
}) {
  const { t } = useTranslation();
  const [counterpartyId, setCounterpartyId] = useState<string>(
    initial.counterpartyId ? String(initial.counterpartyId) : 'none',
  );
  const [description, setDescription] = useState(initial.description ?? '');
  const [category, setCategory] = useState(initial.category ?? 'none');
  const [budget, setBudget] = useState(initial.budget ?? 'none');
  const [notes, setNotes] = useState(initial.notes ?? '');
  const [selectedTags, setSelectedTags] = useState<string[]>(
    initial.tags ?? (defaultTag ? [defaultTag] : []),
  );
  const [attachment, setAttachment] = useState(initial.attachment);

  const filteredAccounts =
    direction === 'withdrawal'
      ? accounts.filter((a) => a.type === 'expense' || a.type === 'asset')
      : accounts.filter((a) => a.type === 'revenue' || a.type === 'asset');

  function handleSave() {
    onSave({
      counterpartyId: counterpartyId === 'none' ? undefined : Number(counterpartyId),
      description: description || undefined,
      category: category === 'none' ? undefined : category,
      budget: budget === 'none' ? undefined : budget,
      tags: selectedTags,
      notes: notes || undefined,
      attachment,
    });
  }

  return (
    <div className="space-y-3 border-t border-border-subtle pt-3">
      <div>
        <p className="mb-1 text-sm text-text-2">{t('results.counterparty')}</p>
        <Select value={counterpartyId} onValueChange={setCounterpartyId}>
          <SelectTrigger className="w-full" aria-label={t('results.counterparty')}>
            <SelectValue placeholder="—" />
          </SelectTrigger>
          <SelectContent position="popper">
            <SelectItem value="none">—</SelectItem>
            {filteredAccounts.map((a) => (
              <SelectItem key={a.id} value={String(a.id)}>
                {a.name} ({a.currency})
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <label className="block text-sm text-text-2">
        {t('results.descriptionField')}
        <Input
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          className="mt-1"
        />
      </label>
      <div>
        <p className="mb-1 text-sm text-text-2">{t('results.tags')}</p>
        <TagsPicker value={selectedTags} options={tags} onChange={setSelectedTags} />
      </div>
      <div>
        <p className="mb-1 text-sm text-text-2">{t('results.category')}</p>
        <Select value={category} onValueChange={setCategory}>
          <SelectTrigger className="w-full" aria-label={t('results.category')}>
            <SelectValue placeholder="—" />
          </SelectTrigger>
          <SelectContent position="popper">
            <SelectItem value="none">—</SelectItem>
            {categories.map((c) => (
              <SelectItem key={c} value={c}>
                {c}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <div>
        <p className="mb-1 text-sm text-text-2">{t('results.budget')}</p>
        <Select value={budget} onValueChange={setBudget}>
          <SelectTrigger className="w-full" aria-label={t('results.budget')}>
            <SelectValue placeholder="—" />
          </SelectTrigger>
          <SelectContent position="popper">
            <SelectItem value="none">—</SelectItem>
            {budgets.map((b) => (
              <SelectItem key={b} value={b}>
                {b}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <label className="block text-sm text-text-2">
        {t('results.notes')}
        <Input value={notes} onChange={(e) => setNotes(e.target.value)} className="mt-1" />
      </label>
      <div>
        <p className="mb-1 text-sm text-text-2">{t('results.attachment')}</p>
        <input
          type="file"
          onChange={(e) => {
            const file = e.target.files?.[0];
            setAttachment(file ? { filename: file.name, file } : undefined);
          }}
          className="text-sm text-text-2"
        />
        {attachment && <p className="text-xs text-text-3">{attachment.filename}</p>}
      </div>
      <div className="flex gap-2 pt-2">
        <Button variant="cancel" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
        <Button onClick={handleSave}>{submitLabel}</Button>
      </div>
    </div>
  );
}
