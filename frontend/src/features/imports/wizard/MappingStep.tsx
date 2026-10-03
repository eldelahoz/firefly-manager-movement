import { useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { UploadResponse } from '@/shared/api/dto';
import { getSettings, putMapping } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';
import { DATE_FORMATS, isAmbiguousFor, parseWithFormat } from '@/shared/lib/dates';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { WizardFooter } from './WizardShell';

export type MappingColumns = {
  date_column: string;
  amount_column: string;
  description_column: string;
  origin_account_column?: string;
  date_format?: string;
};

export function MappingStep({
  probe,
  sampleRows,
  onMapped,
  onBack,
  onCancel,
}: {
  probe: UploadResponse;
  sampleRows?: string[][];
  onMapped: (columns: MappingColumns, response?: { mapping_id: number }) => void;
  onBack: () => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  // Seeded from the last applied mapping so Back from preview preserves the selection.
  const applied = probe.mapping_used;
  const [cols, setCols] = useState<MappingColumns>({
    date_column: applied?.date_column ?? '',
    amount_column: applied?.amount_column ?? '',
    description_column: applied?.description_column ?? '',
    ...(applied?.origin_account_column
      ? { origin_account_column: applied.origin_account_column }
      : {}),
  });
  const [dateFormat, setDateFormat] = useState<string | null>(applied?.date_format ?? null);
  const settings = useQuery({
    queryKey: queryKeys.settings,
    queryFn: getSettings,
    staleTime: 30_000,
  });
  // null = follow the settings default until the user picks explicitly.
  const selectedFormat = dateFormat ?? settings.data?.default_date_format ?? 'auto';
  const put = useMutation({
    mutationFn: () =>
      putMapping(probe.batch_id, {
        headers: probe.headers,
        date_column: cols.date_column,
        amount_column: cols.amount_column,
        description_column: cols.description_column,
        ...(cols.origin_account_column
          ? { origin_account_column: cols.origin_account_column }
          : {}),
        ...(selectedFormat !== 'auto' ? { date_format: selectedFormat } : {}),
      }),
  });
  const complete = cols.date_column && cols.amount_column && cols.description_column;
  const dateIdx = probe.headers.indexOf(cols.date_column);
  const fmt = selectedFormat === 'auto' ? undefined : selectedFormat;
  const raws =
    dateIdx >= 0
      ? (sampleRows ?? [])
          .map((r) => (r[dateIdx] ?? '').trim())
          .filter((v) => v !== '')
          .slice(0, 3)
      : [];
  const ambiguous = raws.some((v) => isAmbiguousFor(v, fmt));
  const fields = [
    ['date_column', t('wizard.dateColumn'), t('wizard.fieldDate')],
    ['amount_column', t('wizard.amountColumn'), t('wizard.fieldAmount')],
    ['description_column', t('wizard.descriptionColumn'), t('wizard.fieldDescription')],
  ] as const;
  const optionalFields = [['origin_account_column', t('wizard.fieldOrigin')]] as const;
  return (
    <div className="flex h-full flex-col">
      <div className="space-y-6">
        {!applied && probe.valid_count === 0 && (
          <p role="status" className="text-sm text-text-2">
            {t('imports.needsMapping')}
          </p>
        )}
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
        <div className="flex items-center gap-3">
          <Select value={selectedFormat} onValueChange={setDateFormat}>
            <SelectTrigger className="w-full max-w-[360px]" aria-label={t('wizard.dateFormat')}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent position="popper">
              <SelectItem value="auto">{t('wizard.dateFormatAuto')}</SelectItem>
              {DATE_FORMATS.map((f) => (
                <SelectItem key={f.value} value={f.value}>
                  {f.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <span className="text-lg text-text-3" aria-hidden="true">
            →
          </span>
          <span className="text-sm text-text-1">{t('wizard.dateFormat')}</span>
        </div>
        <p className="text-xs text-text-3">{t('wizard.dateFormatHint')}</p>
        {raws.length > 0 && (
          <div className="space-y-1 text-sm" data-testid="date-format-preview">
            {raws.map((v) => (
              <div key={v} className="flex gap-2">
                <span className="text-text-3">{v}</span>
                <span className="text-text-3" aria-hidden="true">
                  →
                </span>
                <span>{parseWithFormat(v, fmt) ?? t('wizard.dateFormatInvalid')}</span>
              </div>
            ))}
            {ambiguous && (
              <p className="text-status-missing-txt" role="alert">
                {t('wizard.dateFormatAmbiguous')}
              </p>
            )}
          </div>
        )}
        {optionalFields.map(([key, label]) => (
          <div key={key} className="flex items-center gap-3">
            <Select
              value={cols[key] === '' || cols[key] == null ? 'none' : cols[key]!}
              onValueChange={(v) => setCols({ ...cols, [key]: v === 'none' ? undefined : v })}
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
            <span className="text-sm text-text-1">{label}</span>
          </div>
        ))}
      </div>
      <WizardFooter
        onCancel={onCancel}
        cancelLabel={t('common.cancel')}
        onBack={onBack}
        backLabel={t('common.back')}
        primaryLabel={t('common.next')}
        onPrimary={() =>
          put.mutate(undefined, {
            onSuccess: (res) =>
              onMapped(
                { ...cols, ...(selectedFormat !== 'auto' ? { date_format: selectedFormat } : {}) },
                res,
              ),
          })
        }
        primaryDisabled={!complete || put.isPending}
      />
    </div>
  );
}
