import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { Ff3Account } from '@/shared/api/dto';
import { getAccounts, getTags, getCategories, getBudgets } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';
import type { BulkFields } from './customization';
import { TagsPicker } from './TagsPicker';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@/shared/ui/sheet';

export function BulkPersonalizeSheet({
  count,
  defaultTag,
  onApply,
  onClose,
}: {
  count: number;
  defaultTag: string | null;
  onApply: (fields: BulkFields) => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const [counterpartyId, setCounterpartyId] = useState('none');
  const [description, setDescription] = useState('');
  const [category, setCategory] = useState('none');
  const [budget, setBudget] = useState('none');
  const [notes, setNotes] = useState('');
  const [selectedTags, setSelectedTags] = useState<string[]>(defaultTag ? [defaultTag] : []);
  const [tagsTouched, setTagsTouched] = useState(false);
  const accounts = useQuery({
    queryKey: queryKeys.accounts,
    queryFn: getAccounts,
    staleTime: 300_000,
  });
  const tags = useQuery({ queryKey: queryKeys.tags, queryFn: getTags, staleTime: 300_000 });
  const categories = useQuery({
    queryKey: queryKeys.categories,
    queryFn: getCategories,
    staleTime: 300_000,
  });
  const budgets = useQuery({
    queryKey: queryKeys.budgets,
    queryFn: getBudgets,
    staleTime: 300_000,
  });

  function handleApply() {
    const fields: BulkFields = {};
    if (counterpartyId !== 'none') fields.counterpartyId = Number(counterpartyId);
    if (description) fields.description = description;
    if (category !== 'none') fields.category = category;
    if (budget !== 'none') fields.budget = budget;
    if (notes) fields.notes = notes;
    if (tagsTouched) fields.tags = selectedTags;
    onApply(fields);
  }

  return (
    <Sheet
      open
      onOpenChange={(o) => {
        if (!o) onClose();
      }}
    >
      <SheetContent
        side="right"
        showCloseButton={false}
        className="w-full gap-4 overflow-y-auto p-4 sm:max-w-md"
      >
        <SheetHeader className="p-0">
          <SheetTitle>{t('results.bulkTitle', { count })}</SheetTitle>
        </SheetHeader>
        <p className="text-xs text-text-3">{t('results.bulkHint')}</p>
        <div>
          <p className="mb-1 text-sm text-text-2">{t('results.counterparty')}</p>
          <Select value={counterpartyId} onValueChange={setCounterpartyId}>
            <SelectTrigger className="w-full" aria-label={t('results.counterparty')}>
              <SelectValue placeholder="—" />
            </SelectTrigger>
            <SelectContent position="popper">
              <SelectItem value="none">—</SelectItem>
              {accounts.data?.map((a: Ff3Account) => (
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
          <TagsPicker
            value={selectedTags}
            options={tags.data ?? []}
            onChange={(v) => {
              setSelectedTags(v);
              setTagsTouched(true);
            }}
          />
        </div>
        <div>
          <p className="mb-1 text-sm text-text-2">{t('results.category')}</p>
          <Select value={category} onValueChange={setCategory}>
            <SelectTrigger className="w-full" aria-label={t('results.category')}>
              <SelectValue placeholder="—" />
            </SelectTrigger>
            <SelectContent position="popper">
              <SelectItem value="none">—</SelectItem>
              {(categories.data ?? []).map((c: string) => (
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
              {(budgets.data ?? []).map((b: string) => (
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
        <div className="mt-auto flex gap-2">
          <Button variant="cancel" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button className="flex-1" onClick={handleApply}>
            {t('results.applyToSelection')}
          </Button>
        </div>
      </SheetContent>
    </Sheet>
  );
}
