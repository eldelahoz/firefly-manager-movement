import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { UploadResponse } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { ChevronRight } from 'lucide-react';
import { Button } from '@/shared/ui/button';
import { MoneyText } from '@/shared/ui/MoneyText';
import { WizardFooter } from './WizardShell';
import type { OriginNotice } from './state';

export function PreviewStep({
  probe,
  currency,
  originNotice,
  hideAutoApplied,
  onNext,
  onBack,
  onClose,
}: {
  probe: UploadResponse;
  currency: string;
  originNotice?: OriginNotice;
  hideAutoApplied?: boolean;
  onNext: () => void;
  onBack: () => void;
  onClose: () => void;
}) {
  const { t, i18n } = useTranslation();
  const [invalidOpen, setInvalidOpen] = useState(true);
  return (
    <div className="flex flex-col">
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
      {probe.mapping_used && !hideAutoApplied && (
        <p className="text-xs text-text-3">{t('wizard.autoApplied')}</p>
      )}
      {originNotice?.kind === 'auto' && (
        <p role="status" className="mt-2 text-sm text-status-matched-txt">
          {t('wizard.originAutoSelected', { account: originNotice.accountName })}
        </p>
      )}
      {originNotice?.kind === 'warn' && (
        <div role="alert" className="mt-2 rounded-lg bg-danger-bg/50 p-3 text-sm text-danger">
          {t('wizard.originWarning', { values: originNotice.values.join(', ') })}
        </div>
      )}
      {!probe.mapping_used && probe.valid_count === 0 && probe.invalid_count === 0 && (
        <p className="mt-3 text-sm text-text-2">{t('imports.needsMapping')}</p>
      )}
      {probe.valid.length > 0 && (
        <div className="mt-3 overflow-x-auto rounded-lg border border-border-subtle">
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-canvas text-xs font-semibold text-text-2">
                <th className="px-4 py-2.5 text-left">#</th>
                <th className="px-4 py-2.5 text-left">{t('results.col.date')}</th>
                <th className="px-4 py-2.5 text-right">{t('results.col.amount')}</th>
                <th className="px-4 py-2.5 text-left">{t('results.col.description')}</th>
              </tr>
            </thead>
            <tbody>
              {probe.valid.map((r) => (
                <tr key={r.line_no} className="border-t border-border-subtle">
                  <td className="px-4 py-3 text-[13px]">{r.line_no}</td>
                  <td className="px-4 py-3 text-[13px] whitespace-nowrap">
                    {formatDate(r.date, i18n.language)}
                  </td>
                  <td className="px-4 py-3 text-right text-[13px]">
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
        <section className="mt-3 rounded-lg border border-border-subtle bg-danger-bg/40 p-3">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <button
              type="button"
              onClick={() => setInvalidOpen((o) => !o)}
              aria-expanded={invalidOpen}
              className="flex items-center gap-1.5 rounded-md text-[13px] font-semibold text-danger"
            >
              <ChevronRight
                className={`ffm-chevron size-4 ${invalidOpen ? 'rotate-90' : ''}`}
                aria-hidden="true"
              />
              {t('wizard.invalidRows', { count: probe.invalid_count })}
            </button>
            <Button variant="outline" size="sm" onClick={onBack}>
              {t('wizard.backToMapping')}
            </Button>
          </div>
          {invalidOpen && (
            <ul className="mt-2 max-h-60 space-y-2 overflow-y-auto">
              {probe.invalid.map((r) => (
                <li key={r.line_no} className="text-xs text-text-2">
                  <p>
                    #{r.line_no} · {r.reason}
                  </p>
                  <code className="mt-0.5 block overflow-x-auto rounded bg-surface px-2 py-1 font-mono whitespace-pre text-text-1">
                    {r.raw}
                  </code>
                </li>
              ))}
            </ul>
          )}
        </section>
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
