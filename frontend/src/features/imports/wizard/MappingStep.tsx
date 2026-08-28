import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { UploadResponse } from '@/shared/api/dto';
import { putMapping } from '@/shared/api/endpoints';

export type MappingColumns = {
  date_column: string;
  amount_column: string;
  description_column: string;
};

export function MappingStep({
  probe,
  onMapped,
}: {
  probe: UploadResponse;
  onMapped: (columns: MappingColumns) => void;
}) {
  const { t } = useTranslation();
  const [cols, setCols] = useState<MappingColumns>({
    date_column: '',
    amount_column: '',
    description_column: '',
  });
  const put = useMutation({
    mutationFn: () => putMapping(probe.batch_id, { headers: probe.headers, ...cols }),
  });
  const complete = cols.date_column && cols.amount_column && cols.description_column;
  const fields = [
    ['date_column', t('wizard.dateColumn')],
    ['amount_column', t('wizard.amountColumn')],
    ['description_column', t('wizard.descriptionColumn')],
  ] as const;
  return (
    <div className="space-y-4">
      {fields.map(([key, label]) => (
        <label key={key} className="block text-sm text-text-2">
          {label}
          <select
            value={cols[key]}
            onChange={(e) => setCols({ ...cols, [key]: e.target.value })}
            className="mt-1 h-11 w-full rounded-lg border border-border-subtle bg-surface px-2 text-text-1"
          >
            <option value="">—</option>
            {probe.headers.map((h) => (
              <option key={h} value={h}>
                {h}
              </option>
            ))}
          </select>
        </label>
      ))}
      <button
        disabled={!complete || put.isPending}
        onClick={() =>
          put.mutate(undefined, {
            onSuccess: () => onMapped(cols),
          })
        }
        className="h-11 w-full rounded-lg bg-accent font-medium text-white disabled:opacity-50"
      >
        {t('common.next')}
      </button>
    </div>
  );
}
