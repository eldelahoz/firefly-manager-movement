import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Ff3Account } from '@/shared/api/dto';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';

const MAX_BYTES = 10 * 1024 * 1024;

export function UploadStep({
  accounts,
  accountId,
  onAccount,
  onFile,
  uploading,
}: {
  accounts: Ff3Account[] | undefined;
  accountId: string;
  onAccount: (id: string, currency: string) => void;
  onFile: (f: File) => void;
  uploading: boolean;
}) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);
  const [tooLarge, setTooLarge] = useState(false);
  const currency = accounts?.find((a) => a.id === Number(accountId))?.currency;
  const ready = !!accountId;
  function handle(f: File | undefined) {
    if (!f || !ready) return;
    const oversize = f.size > MAX_BYTES;
    setTooLarge(oversize);
    if (!oversize) onFile(f);
  }
  return (
    <div className="space-y-4">
      <label className="block text-sm text-text-2">
        {t('wizard.account')}
        <Select
          value={accountId === '' ? 'none' : accountId}
          onValueChange={(v) => {
            const id = v === 'none' ? '' : v;
            onAccount(id, accounts?.find((a) => a.id === Number(id))?.currency ?? '');
          }}
        >
          <SelectTrigger className="mt-1 w-full" aria-label={t('wizard.account')}>
            <SelectValue placeholder="—" />
          </SelectTrigger>
          <SelectContent position="popper">
            <SelectItem value="none">—</SelectItem>
            {accounts?.map((a) => (
              <SelectItem key={a.id} value={String(a.id)}>
                {a.name} ({a.currency})
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </label>
      {currency && (
        <p className="text-sm text-text-3">{t('wizard.accountCurrency', { currency })}</p>
      )}
      <div
        role="button"
        tabIndex={ready ? 0 : -1}
        aria-disabled={!ready}
        onClick={() => ready && inputRef.current?.click()}
        onKeyDown={(e) => e.key === 'Enter' && ready && inputRef.current?.click()}
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          e.preventDefault();
          handle(e.dataTransfer.files[0]);
        }}
        className={`flex h-48 flex-col items-center justify-center rounded-xl border-2 border-dashed border-border-strong bg-canvas text-text-2 ${ready ? 'cursor-pointer' : 'pointer-events-none opacity-50'}`}
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
      {uploading && <p aria-live="polite">{t('wizard.uploading')}</p>}
    </div>
  );
}
