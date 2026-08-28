import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { getAccounts, postApply } from '@/shared/api/endpoints';
import type { ApplyReport } from '@/shared/api/dto';
import { queryKeys } from '@/shared/lib/keys';

export function ApplyPanel({
  batchId,
  selected,
  onDone,
}: {
  batchId: number;
  selected: number[];
  onDone: (reports: ApplyReport[]) => void;
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
    <div
      role="dialog"
      aria-label={t('results.apply')}
      className="fixed inset-x-0 bottom-0 z-40 rounded-t-xl border-t border-border-subtle bg-surface p-4 shadow-lg md:inset-y-0 md:right-0 md:left-auto md:w-96 md:rounded-none md:border-l"
    >
      <h2 className="font-semibold">
        {t('results.apply')} — {selected.length}
      </h2>
      <label className="mt-3 block text-sm text-text-2">
        {t('results.counterparty')}
        <select
          value={counterparty}
          onChange={(e) => setCounterparty(e.target.value)}
          className="mt-1 h-11 w-full rounded-lg border border-border-subtle bg-surface px-2 text-text-1"
        >
          <option value="">—</option>
          {accounts.data?.map((a) => (
            <option key={a.id} value={String(a.id)}>
              {a.name} ({a.currency})
            </option>
          ))}
        </select>
      </label>
      <label className="mt-3 block text-sm text-text-2">
        {t('results.category')}
        <input
          value={category}
          onChange={(e) => setCategory(e.target.value)}
          className="mt-1 h-11 w-full rounded-lg border border-border-subtle bg-surface px-2"
        />
      </label>
      <label className="mt-3 block text-sm text-text-2">
        {t('results.budget')}
        <input
          value={budget}
          onChange={(e) => setBudget(e.target.value)}
          className="mt-1 h-11 w-full rounded-lg border border-border-subtle bg-surface px-2"
        />
      </label>
      <label className="mt-3 block text-sm text-text-2">
        {t('results.notes')}
        <input
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          className="mt-1 h-11 w-full rounded-lg border border-border-subtle bg-surface px-2"
        />
      </label>
      <div className="mt-4 flex gap-2">
        <button
          onClick={() => onDone([])}
          className="h-11 rounded-lg border border-border-subtle bg-surface px-4 font-medium text-text-2"
        >
          {t('common.cancel')}
        </button>
        <button
          disabled={!ready}
          onClick={() => apply.mutate()}
          className="h-11 flex-1 rounded-lg bg-accent font-medium text-white disabled:opacity-50"
        >
          {t('results.apply')}
        </button>
      </div>
    </div>
  );
}
