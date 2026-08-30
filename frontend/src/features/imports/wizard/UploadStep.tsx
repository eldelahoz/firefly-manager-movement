import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Ff3Account } from '@/shared/api/dto';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { WizardFooter } from './WizardShell';

const MAX_BYTES = 10 * 1024 * 1024;

export function UploadStep({
  accounts,
  accountId,
  onAccount,
  onFile,
  uploading,
  onCancel,
  error,
}: {
  accounts: Ff3Account[] | undefined;
  accountId: string;
  onAccount: (id: string, currency: string) => void;
  onFile: (f: File) => void;
  uploading: boolean;
  onCancel: () => void;
  error: string | null;
}) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);
  const [tooLarge, setTooLarge] = useState(false);
  const [dragging, setDragging] = useState(false);
  const currency = accounts?.find((a) => a.id === Number(accountId))?.currency;
  const ready = !!accountId;
  function handle(f: File | undefined) {
    if (!f || !ready) return;
    const oversize = f.size > MAX_BYTES;
    setTooLarge(oversize);
    if (!oversize) onFile(f);
  }
  function pick() {
    if (ready) inputRef.current?.click();
  }
  const dropzoneBase =
    'flex cursor-pointer flex-col items-center justify-center gap-1 rounded-xl border-2 border-dashed';
  return (
    <div className="flex h-full flex-col">
      <div>
        <label className="text-[13px] text-text-2">
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
          <p className="mt-1 text-xs text-text-3">{t('wizard.accountCurrency', { currency })}</p>
        )}
      </div>
      {error && (
        <div
          role="alert"
          className="mt-4 flex items-center gap-2 rounded-lg bg-status-extra-bg p-3"
        >
          <span className="text-lg leading-none font-semibold text-destructive">!</span>
          <span className="text-sm font-medium text-status-extra-txt">{error}</span>
        </div>
      )}
      {tooLarge ? (
        <div
          className={`mt-4 flex flex-1 flex-col items-center justify-center gap-2 rounded-xl bg-status-extra-bg/50 ${ready ? '' : 'pointer-events-none opacity-50'}`}
        >
          <span className="flex size-12 items-center justify-center rounded-full bg-destructive text-3xl font-semibold text-white">
            !
          </span>
          <p className="text-base font-semibold text-destructive">{t('wizard.fileTooLarge')}</p>
          <p className="text-xs text-text-2">{t('wizard.maxSize')}</p>
        </div>
      ) : uploading ? (
        <div
          className="mt-4 flex flex-1 flex-col items-center justify-center gap-3"
          aria-live="polite"
        >
          <p className="text-sm font-semibold">{t('wizard.uploading')}</p>
          <div className="h-2 w-full max-w-md overflow-hidden rounded bg-border-subtle">
            <div className="h-full w-1/3 animate-pulse rounded bg-accent" />
          </div>
        </div>
      ) : (
        <div
          role="button"
          tabIndex={ready ? 0 : -1}
          aria-disabled={!ready}
          onClick={pick}
          onKeyDown={(e) => e.key === 'Enter' && pick()}
          onDragOver={(e) => {
            e.preventDefault();
            if (ready) setDragging(true);
          }}
          onDragLeave={() => setDragging(false)}
          onDrop={(e) => {
            e.preventDefault();
            setDragging(false);
            handle(e.dataTransfer.files[0]);
          }}
          className={`mt-4 ${error ? 'h-28' : 'h-full min-h-[280px] flex-1'} ${dropzoneBase} ${
            dragging ? 'border-accent bg-accent-soft/60' : 'border-border-subtle bg-surface'
          } ${ready ? '' : 'pointer-events-none opacity-50'}`}
        >
          {dragging ? (
            <p className="text-base font-semibold text-accent">{t('wizard.dropToUpload')}</p>
          ) : (
            <>
              <span className="text-5xl leading-none text-text-3" aria-hidden="true">
                +
              </span>
              <p className="text-base font-semibold">{t('wizard.dropTitle')}</p>
              {error ? (
                <p className="text-sm text-text-2">{t('wizard.anotherFile')}</p>
              ) : (
                <>
                  <p className="text-sm text-text-2">
                    {t('wizard.or')}{' '}
                    <span className="font-medium text-accent underline-offset-2">
                      {t('wizard.browse')}
                    </span>
                  </p>
                  <p className="text-xs text-text-3">{t('wizard.maxSize')}</p>
                </>
              )}
            </>
          )}
        </div>
      )}
      <input
        ref={inputRef}
        type="file"
        accept=".csv"
        className="hidden"
        onChange={(e) => handle(e.target.files?.[0])}
      />
      <WizardFooter
        onCancel={onCancel}
        cancelLabel={t('common.cancel')}
        onBack={() => {}}
        backDisabled
        backLabel={t('common.back')}
        primaryLabel={t('common.next')}
        primaryDisabled
      />
    </div>
  );
}
