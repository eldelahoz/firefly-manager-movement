import { useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import type { UploadResponse } from '@/shared/api/dto';
import { getSettings, putMapping } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';
import { DATE_FORMATS, formatDate, isAmbiguousFor, parseWithFormat } from '@/shared/lib/dates';
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
  /** Raw rows of the staged file; undefined on resume (the file is not kept after upload). */
  sampleRows?: string[][];
  onMapped: (columns: MappingColumns, response?: { mapping_id: number }) => void;
  onBack: () => void;
  onCancel: () => void;
}) {
  const { t, i18n } = useTranslation();
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
  const fmt = selectedFormat === 'auto' ? undefined : selectedFormat;
  // First non-empty raw values of a column, from the staged file (empty on resume: no file).
  const rawsOf = (column: string | undefined, n: number) => {
    const idx = column ? probe.headers.indexOf(column) : -1;
    if (idx < 0) return [];
    return (sampleRows ?? [])
      .map((r) => (r[idx] ?? '').trim())
      .filter((v) => v !== '')
      .slice(0, n);
  };
  const ambiguous = rawsOf(cols.date_column, 3).some((v) => isAmbiguousFor(v, fmt));
  // Dates only: amounts stay raw here; the preview shows the backend-parsed result (reviewer M4 #1).
  const parseDate = (v: string) => {
    const iso = parseWithFormat(v, fmt);
    return iso ? formatDate(iso, i18n.language) : t('wizard.dateFormatInvalid');
  };
  const samples = (key: keyof MappingColumns, field: string) => {
    const raws = rawsOf(cols[key], 2);
    if (raws.length === 0) return null;
    return (
      <ul
        aria-label={t('wizard.samplesFor', { field })}
        data-testid={key === 'date_column' ? 'date-format-preview' : undefined}
        className="mt-1.5 space-y-0.5 text-xs text-text-2"
      >
        {raws.map((v, i) => (
          <li key={i} className="flex min-w-0 gap-2">
            <span className="truncate text-text-3">{v}</span>
            {key === 'date_column' && (
              <>
                <span className="text-text-3" aria-hidden="true">
                  →
                </span>
                <span className="whitespace-nowrap">{parseDate(v)}</span>
              </>
            )}
          </li>
        ))}
      </ul>
    );
  };
  const columnSelect = (key: keyof MappingColumns, label: string, required: boolean) => (
    <Select
      value={cols[key] ? cols[key]! : 'none'}
      onValueChange={(v) =>
        setCols({ ...cols, [key]: v === 'none' ? (required ? '' : undefined) : v })
      }
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
  );
  const fieldRow = (control: React.ReactNode, field: React.ReactNode) => (
    <div className="flex items-center gap-3">
      {control}
      <span className="text-lg text-text-3" aria-hidden="true">
        →
      </span>
      <span className="shrink-0 text-sm text-text-1">{field}</span>
    </div>
  );
  const required = (field: string) => (
    <>
      {field} <span className="font-semibold text-destructive">*</span>
    </>
  );
  return (
    <div className="flex flex-col">
      <div className="space-y-5">
        {!applied && probe.valid_count === 0 && (
          <p role="status" className="text-sm text-text-2">
            {t('imports.needsMapping')}
          </p>
        )}
        {sampleRows === undefined && (
          <p className="text-xs text-text-3">{t('wizard.samplesUnavailable')}</p>
        )}
        <div className="space-y-3">
          {fieldRow(
            columnSelect('date_column', t('wizard.dateColumn'), true),
            required(t('wizard.fieldDate')),
          )}
          {fieldRow(
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
            </Select>,
            t('wizard.dateFormat'),
          )}
          <p className="text-xs text-text-3">{t('wizard.dateFormatHint')}</p>
          {samples('date_column', t('wizard.fieldDate'))}
          {ambiguous && (
            <p className="text-sm text-status-missing-txt" role="alert">
              {t('wizard.dateFormatAmbiguous')}
            </p>
          )}
        </div>
        <div>
          {fieldRow(
            columnSelect('amount_column', t('wizard.amountColumn'), true),
            required(t('wizard.fieldAmount')),
          )}
          {samples('amount_column', t('wizard.fieldAmount'))}
        </div>
        <div>
          {fieldRow(
            columnSelect('description_column', t('wizard.descriptionColumn'), true),
            required(t('wizard.fieldDescription')),
          )}
          {samples('description_column', t('wizard.fieldDescription'))}
        </div>
        <div>
          {fieldRow(
            columnSelect('origin_account_column', t('wizard.fieldOrigin'), false),
            t('wizard.fieldOrigin'),
          )}
          {samples('origin_account_column', t('wizard.fieldOrigin'))}
        </div>
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
