import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { UploadResponse } from '@/shared/api/dto';
import { putMapping } from '@/shared/api/endpoints';
import { Button } from '@/shared/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';

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
        <div key={key}>
          <p className="mb-1 text-sm text-text-2">{label}</p>
          <Select
            value={cols[key] === '' ? 'none' : cols[key]}
            onValueChange={(v) => setCols({ ...cols, [key]: v === 'none' ? '' : v })}
          >
            <SelectTrigger className="w-full" aria-label={label}>
              <SelectValue placeholder="—" />
            </SelectTrigger>
            <SelectContent position="popper">
              <SelectItem value="none">—</SelectItem>
              {probe.headers.map((h) => (
                <SelectItem key={h} value={h}>
                  {h}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      ))}
      <Button
        disabled={!complete || put.isPending}
        onClick={() =>
          put.mutate(undefined, {
            onSuccess: () => onMapped(cols),
          })
        }
        className="w-full"
      >
        {t('common.next')}
      </Button>
    </div>
  );
}
