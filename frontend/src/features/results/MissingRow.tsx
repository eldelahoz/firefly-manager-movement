import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import type { MovementDTO } from '@/shared/api/dto';
import { postIgnore } from '@/shared/api/endpoints';
import { formatDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { MoneyText } from '@/shared/ui/MoneyText';
import { StatusPill } from '@/shared/ui/StatusPill';

export function MissingRow({
  movement,
  currency,
  batchId,
  selected,
  onToggle,
}: {
  movement: MovementDTO;
  currency: string;
  batchId: number;
  selected?: boolean;
  onToggle?: (id: number) => void;
}) {
  const { t, i18n } = useTranslation();
  const qc = useQueryClient();
  const ignore = useMutation({
    mutationFn: () => postIgnore(batchId, movement.id),
    onSuccess: () => {
      toast.success(t('status.ignored'));
      void qc.invalidateQueries({ queryKey: queryKeys.results(batchId) });
      void qc.invalidateQueries({ queryKey: queryKeys.import(batchId) });
    },
  });
  return (
    <div className="flex items-center gap-3 border-b border-border-subtle py-3 text-sm">
      <input
        type="checkbox"
        aria-label={movement.description}
        checked={selected ?? false}
        onChange={() => onToggle?.(movement.id)}
        className="size-5"
      />
      <div className="min-w-0 flex-1">
        <p className="truncate">{movement.description}</p>
        <p className="text-xs text-text-3">
          #{movement.line_no} · {formatDate(movement.date, i18n.language)}
        </p>
      </div>
      <MoneyText cents={movement.amount_cents} currency={currency} />
      {movement.status === 'ignored' ? (
        <StatusPill status="ignored" />
      ) : (
        <button
          onClick={() => ignore.mutate()}
          disabled={ignore.isPending}
          className="rounded-lg px-2 py-1 text-xs text-text-2 hover:bg-canvas"
        >
          {t('results.ignore')}
        </button>
      )}
    </div>
  );
}
