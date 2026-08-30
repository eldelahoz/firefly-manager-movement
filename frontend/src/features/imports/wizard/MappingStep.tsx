import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { UploadResponse } from '@/shared/api/dto';
import { putMapping } from '@/shared/api/endpoints';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { WizardFooter } from './WizardShell';

export type MappingColumns = {
  date_column: string;
  amount_column: string;
  description_column: string;
};

export function MappingStep({
  probe,
  onMapped,
  onBack,
  onCancel,
}: {
  probe: UploadResponse;
  onMapped: (columns: MappingColumns) => void;
  onBack: () => void;
  onCancel: () => void;
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
    ['date_column', t('wizard.dateColumn'), t('wizard.fieldDate')],
    ['amount_column', t('wizard.amountColumn'), t('wizard.fieldAmount')],
    ['description_column', t('wizard.descriptionColumn'), t('wizard.fieldDescription')],
  ] as const;
  return (
    <div className="flex h-full flex-col">
      <div className="space-y-6">
        {fields.map(([key, label, field]) => (
          <div key={key} className="flex items-center gap-3">
            <Select
              value={cols[key] === '' ? 'none' : cols[key]}
              onValueChange={(v) => setCols({ ...cols, [key]: v === 'none' ? '' : v })}
            >
              <SelectTrigger className="w-full max-w-[360px]" aria-label={label}>
                <SelectValue placeholder={t('wizard.sourceColumn')} />
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
            <span className="text-lg text-text-3" aria-hidden="true">
              →
            </span>
            <span className="text-sm text-text-1">
              {field} <span className="font-semibold text-destructive">*</span>
            </span>
          </div>
        ))}
      </div>
      <WizardFooter
        onCancel={onCancel}
        cancelLabel={t('common.cancel')}
        onBack={onBack}
        backLabel={t('common.back')}
        primaryLabel={t('common.next')}
        onPrimary={() => put.mutate(undefined, { onSuccess: () => onMapped(cols) })}
        primaryDisabled={!complete || put.isPending}
      />
    </div>
  );
}
