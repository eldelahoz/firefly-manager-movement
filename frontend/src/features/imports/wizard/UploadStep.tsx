import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

const MAX_BYTES = 10 * 1024 * 1024;
const CURRENCIES = ['EUR', 'USD', 'GBP', 'CHF'];

export function UploadStep({
  currency,
  onCurrency,
  onFile,
  uploading,
}: {
  currency: string;
  onCurrency: (c: string) => void;
  onFile: (f: File) => void;
  uploading: boolean;
}) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);
  const [tooLarge, setTooLarge] = useState(false);
  function handle(f: File | undefined) {
    if (!f) return;
    const oversize = f.size > MAX_BYTES;
    setTooLarge(oversize);
    if (!oversize) onFile(f);
  }
  return (
    <div className="space-y-4">
      <div
        role="button"
        tabIndex={0}
        onClick={() => inputRef.current?.click()}
        onKeyDown={(e) => e.key === 'Enter' && inputRef.current?.click()}
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          e.preventDefault();
          handle(e.dataTransfer.files[0]);
        }}
        className="flex h-48 cursor-pointer flex-col items-center justify-center rounded-xl border-2 border-dashed border-border-strong bg-canvas text-text-2"
      >
        <p>{t('wizard.dropHere')}</p>
      </div>
      <input
        ref={inputRef}
        type="file"
        accept=".csv"
        className="hidden"
        onChange={(e) => handle(e.target.files?.[0])}
      />
      {tooLarge && (
        <p role="alert" className="text-sm font-medium text-status-extra-txt">
          {t('wizard.fileTooLarge')}
        </p>
      )}
      <label className="flex items-center gap-2 text-sm text-text-2">
        {t('wizard.currency')}
        <select
          value={currency}
          onChange={(e) => onCurrency(e.target.value)}
          className="h-11 rounded-lg border border-border-subtle bg-surface px-2"
        >
          {CURRENCIES.map((c) => (
            <option key={c}>{c}</option>
          ))}
        </select>
      </label>
      {uploading && <p aria-live="polite">{t('wizard.uploading')}</p>}
    </div>
  );
}
