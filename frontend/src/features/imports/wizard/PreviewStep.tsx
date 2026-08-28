import { useTranslation } from 'react-i18next';
import type { UploadResponse } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Table, Td, Tr } from '@/shared/ui/Table';

export function PreviewStep({
  probe,
  currency,
  onNext,
  onBack,
}: {
  probe: UploadResponse;
  currency: string;
  onNext: () => void;
  onBack: () => void;
}) {
  const { t, i18n } = useTranslation();
  return (
    <div className="space-y-4">
      <p className="text-sm text-text-2">
        {t('wizard.delimiter')}:{' '}
        <code className="rounded bg-canvas px-1">{probe.detected_delimiter}</code>
      </p>
      {probe.valid.length > 0 && (
        <>
          <p className="text-sm text-text-3">
            {t('wizard.showingFirst', { total: probe.valid_count })}
          </p>
          <Table
            head={[
              '#',
              t('imports.createdAt'),
              t('wizard.amountColumn'),
              t('wizard.descriptionColumn'),
            ]}
          >
            {probe.valid.map((r) => (
              <Tr key={r.line_no}>
                <Td>{r.line_no}</Td>
                <Td>{formatDate(r.date, i18n.language)}</Td>
                <Td>
                  <MoneyText cents={r.amount_cents} currency={currency} />
                </Td>
                <Td className="max-w-48 truncate">{r.description}</Td>
              </Tr>
            ))}
          </Table>
        </>
      )}
      {probe.mapping_used && <p className="text-xs text-text-3">{t('wizard.autoApplied')}</p>}
      {probe.invalid_count > 0 && (
        <section>
          <h3 className="font-semibold text-status-missing-txt">
            {t('wizard.invalidRows', { count: probe.invalid_count })}
          </h3>
          <ul className="mt-2 space-y-1 text-sm">
            {probe.invalid.map((r) => (
              <li
                key={r.line_no}
                className="rounded-lg bg-status-missing-bg px-3 py-2 text-status-missing-txt"
              >
                #{r.line_no} — {r.reason}
              </li>
            ))}
          </ul>
        </section>
      )}
      <div className="flex justify-between">
        <button onClick={onBack} className="h-11 rounded-lg px-4 text-text-2">
          {t('common.back')}
        </button>
        <button onClick={onNext} className="h-11 rounded-lg bg-accent px-4 font-medium text-white">
          {t('common.next')}
        </button>
      </div>
    </div>
  );
}
