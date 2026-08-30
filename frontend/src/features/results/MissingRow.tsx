import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import type { MovementDTO } from '@/shared/api/dto';
import { postIgnore } from '@/shared/api/endpoints';
import { formatDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Checkbox } from '@/shared/ui/checkbox';
import { StatusPill } from '@/shared/ui/StatusPill';

export function MissingRow({
  movement,
  currency,
  batchId,
  selected,
  onToggle,
  report,
}: {
  movement: MovementDTO;
  currency: string;
  batchId: number;
  selected?: boolean;
  onToggle?: (id: number) => void;
  report?: { outcome: 'created' | 'already_created' | 'failed'; reason?: string };
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
      <Checkbox
        aria-label={movement.description}
        checked={selected ?? false}
        onCheckedChange={() => onToggle?.(movement.id)}
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
        <Button
          variant="ghost"
          size="sm"
          onClick={() => ignore.mutate()}
          disabled={ignore.isPending}
        >
          {t('results.ignore')}
        </Button>
      )}
      {report && (
        <span title={report.reason ?? ''}>
          <StatusPill status={report.outcome} />
        </span>
      )}
    </div>
  );
}
