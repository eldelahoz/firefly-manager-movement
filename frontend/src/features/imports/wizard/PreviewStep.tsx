import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { UploadResponse } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { MoneyText } from '@/shared/ui/MoneyText';
import { WizardFooter } from './WizardShell';

export function PreviewStep({
  probe,
  currency,
  onNext,
  onBack,
  onClose,
}: {
  probe: UploadResponse;
  currency: string;
  onNext: () => void;
  onBack: () => void;
  onClose: () => void;
}) {
  const { t, i18n } = useTranslation();
  const [invalidOpen, setInvalidOpen] = useState(true);
  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2">
        <p className="text-sm font-semibold">
          {t('wizard.validRows', { count: probe.valid_count })}
        </p>
        <span className="rounded-full bg-accent-soft px-2.5 py-0.5 text-[11px] font-medium text-accent">
          {t('status.parsed')}
        </span>
        <span className="ml-auto text-xs text-text-3">
          {t('wizard.delimiter')}: {probe.detected_delimiter}
        </span>
      </div>
      {probe.mapping_used && <p className="text-xs text-text-3">{t('wizard.autoApplied')}</p>}
      {!probe.mapping_used && probe.valid_count === 0 && probe.invalid_count === 0 && (
        <p className="mt-3 text-sm text-text-2">{t('imports.needsMapping')}</p>
      )}
      {probe.valid.length > 0 && (
        <div className="mt-3 overflow-hidden rounded-lg border border-border-subtle">
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-canvas text-xs font-semibold text-text-2">
                <th className="px-4 py-2.5 text-left">#</th>
                <th className="px-4 py-2.5 text-left">{t('imports.createdAt')}</th>
                <th className="px-4 py-2.5 text-left">{t('wizard.amountColumn')}</th>
                <th className="px-4 py-2.5 text-left">{t('wizard.descriptionColumn')}</th>
              </tr>
            </thead>
            <tbody>
              {probe.valid.map((r) => (
                <tr key={r.line_no} className="border-t border-border-subtle">
                  <td className="px-4 py-3 text-[13px]">{r.line_no}</td>
                  <td className="px-4 py-3 text-[13px]">{formatDate(r.date, i18n.language)}</td>
                  <td className="px-4 py-3 text-[13px]">
                    <MoneyText cents={r.amount_cents} currency={currency} />
                  </td>
                  <td className="max-w-48 truncate px-4 py-3 text-[13px]">{r.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {probe.invalid_count > 0 && (
        <div className="mt-3 rounded-lg bg-status-extra-bg/50 p-4">
          <button
            type="button"
            onClick={() => setInvalidOpen((o) => !o)}
            aria-expanded={invalidOpen}
            className="text-[13px] font-semibold text-status-extra-txt"
          >
            {t('wizard.invalidRows', { count: probe.invalid_count })} {invalidOpen ? '−' : '+'}
          </button>
          {invalidOpen && (
            <ul className="mt-2 space-y-1">
              {probe.invalid.map((r) => (
                <li key={r.line_no} className="text-xs text-text-2">
                  #{r.line_no} — {r.reason}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      <WizardFooter
        onCancel={onClose}
        cancelLabel={t('common.cancel')}
        onBack={onBack}
        backLabel={t('common.back')}
        primaryLabel={t('common.next')}
        onPrimary={onNext}
      />
    </div>
  );
}
