import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { getAccounts, postApply } from '@/shared/api/endpoints';
import type { ApplyReport } from '@/shared/api/dto';
import { queryKeys } from '@/shared/lib/keys';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { Sheet, SheetContent, SheetHeader, SheetTitle } from '@/shared/ui/sheet';

export function ApplyPanel({
  batchId,
  selected,
  onDone,
  onClose,
}: {
  batchId: number;
  selected: number[];
  onDone: (reports: ApplyReport[]) => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [counterparty, setCounterparty] = useState('');
  const [category, setCategory] = useState('');
  const [budget, setBudget] = useState('');
  const [notes, setNotes] = useState('');
  const accounts = useQuery({
    queryKey: queryKeys.accounts,
    queryFn: getAccounts,
    staleTime: 300_000,
  });
  const apply = useMutation({
    mutationFn: () =>
      postApply(batchId, {
        movement_ids: selected,
        counterparty_ff3_id: Number(counterparty),
        category: category || undefined,
        budget: budget || undefined,
        notes: notes || undefined,
      }),
    onSuccess: (res) => {
      toast.success(
        t('results.applyRecap', {
          created: res.created_count,
          already: res.already_created_count,
          failed: res.failed_count,
        }),
      );
      void qc.invalidateQueries({ queryKey: queryKeys.results(batchId) });
      void qc.invalidateQueries({ queryKey: queryKeys.import(batchId) });
      void qc.invalidateQueries({ queryKey: queryKeys.imports });
      onDone(res.reports);
    },
    onError: (e) => toast.error(e.message),
  });
  const ready = selected.length > 0 && !!counterparty && !apply.isPending;
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
          <SheetTitle>
            {t('results.apply')} — {selected.length}
          </SheetTitle>
        </SheetHeader>
        <div>
          <p className="mb-1 text-sm text-text-2">{t('results.counterparty')}</p>
          <Select
            value={counterparty === '' ? 'none' : counterparty}
            onValueChange={(v) => setCounterparty(v === 'none' ? '' : v)}
          >
            <SelectTrigger className="w-full" aria-label={t('results.counterparty')}>
              <SelectValue placeholder="—" />
            </SelectTrigger>
            <SelectContent position="popper">
              <SelectItem value="none">—</SelectItem>
              {accounts.data?.map((a) => (
                <SelectItem key={a.id} value={String(a.id)}>
                  {a.name} ({a.currency})
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <label className="block text-sm text-text-2">
          {t('results.category')}
          <Input value={category} onChange={(e) => setCategory(e.target.value)} className="mt-1" />
        </label>
        <label className="block text-sm text-text-2">
          {t('results.budget')}
          <Input value={budget} onChange={(e) => setBudget(e.target.value)} className="mt-1" />
        </label>
        <label className="block text-sm text-text-2">
          {t('results.notes')}
          <Input value={notes} onChange={(e) => setNotes(e.target.value)} className="mt-1" />
        </label>
        <div className="mt-auto flex gap-2">
          <Button variant="cancel" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button
            disabled={!ready}
            className="flex-1"
            onClick={() => {
              apply.mutate();
            }}
          >
            {t('results.apply')}
          </Button>
        </div>
      </SheetContent>
    </Sheet>
  );
}
