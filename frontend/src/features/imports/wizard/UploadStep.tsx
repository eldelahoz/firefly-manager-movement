import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Ff3Account } from '@/shared/api/dto';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { LoaderCircle } from 'lucide-react';
import { WizardFooter } from './WizardShell';

const MAX_BYTES = 10 * 1024 * 1024;

export function UploadStep({
  accounts,
  accountId,
  onAccount,
  onFile,
  onNext,
  file,
  onRemoveFile,
  uploading,
  onCancel,
  error,
}: {
  accounts: Ff3Account[] | undefined;
  accountId: string;
  onAccount: (id: string, currency: string) => void;
  onFile: (f: File) => void;
  onNext: () => void;
  file?: File | null;
  onRemoveFile?: () => void;
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

  // Reset file input when file is removed so the same file can be re-selected
  useEffect(() => {
    if (!file && inputRef.current) {
      inputRef.current.value = '';
    }
  }, [file]);

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
    <div className="flex h-full flex-col relative">
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
          className="mt-4 flex items-center gap-2 rounded-lg bg-status-extra-bg p-3 animate-in fade-in duration-200"
        >
          <span className="text-lg leading-none font-semibold text-destructive">!</span>
          <span className="text-sm font-medium text-status-extra-txt">{error}</span>
        </div>
      )}

      {tooLarge ? (
        <div className="mt-4 flex flex-1 flex-col items-center justify-center gap-2 rounded-xl bg-status-extra-bg/50 animate-in fade-in duration-200">
          <span className="flex size-12 items-center justify-center rounded-full bg-destructive text-3xl font-semibold text-white">
            !
          </span>
          <p className="text-base font-semibold text-destructive">{t('wizard.fileTooLarge')}</p>
          <p className="text-xs text-text-2">{t('wizard.maxSize')}</p>
        </div>
      ) : uploading ? (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-surface/90 backdrop-blur-sm animate-in fade-in duration-200"
          role="status"
          aria-live="polite"
          aria-busy="true"
        >
          <div className="flex flex-col items-center gap-5 p-8 rounded-xl bg-surface border border-border-subtle shadow-xl animate-in zoom-in-95 fade-in duration-200">
            <LoaderCircle className="size-12 text-accent animate-spin" aria-hidden="true" />
            <div className="flex flex-col items-center gap-1">
              <p className="text-base font-semibold text-text-1 text-center">
                {t('wizard.uploading')}
              </p>
              <p className="text-sm text-text-3 text-center">{t('wizard.uploadingFile')}</p>
            </div>
          </div>
        </div>
      ) : ready && file && !error ? (
        // File already selected and no error: show file name with change/remove options
        <div className="mt-4 flex flex-1 flex-col items-center justify-center gap-3 rounded-xl border border-border-subtle bg-surface p-6 animate-in slide-in-from-bottom-4 fade-in duration-300 ease-out">
          <div className="flex items-center gap-3">
            <span className="text-3xl" aria-hidden="true">
              📄
            </span>
            <span className="text-sm font-medium text-text-1">{file.name}</span>
          </div>
          <p className="text-xs text-text-3">{(file.size / 1024).toFixed(1)} KB</p>
          <div className="flex gap-2 mt-2">
            <button
              type="button"
              onClick={pick}
              className="px-3 py-1.5 text-xs font-medium text-accent bg-accent-soft/60 rounded-md hover:bg-accent-soft transition-colors"
            >
              {t('wizard.changeFile')}
            </button>
            {onRemoveFile && (
              <button
                type="button"
                onClick={() => {
                  onRemoveFile();
                  setTooLarge(false);
                }}
                className="px-3 py-1.5 text-xs font-medium text-destructive bg-destructive/10 rounded-md hover:bg-destructive/20 transition-colors"
              >
                {t('wizard.removeFile')}
              </button>
            )}
          </div>
        </div>
      ) : (
        ready && (
          <div
            role="button"
            tabIndex={0}
            onClick={pick}
            onKeyDown={(e) => e.key === 'Enter' && pick()}
            onDragOver={(e) => {
              e.preventDefault();
              setDragging(true);
            }}
            onDragLeave={() => setDragging(false)}
            onDrop={(e) => {
              e.preventDefault();
              setDragging(false);
              handle(e.dataTransfer.files[0]);
            }}
            className={`mt-4 ${error ? 'h-28' : 'h-full min-h-[280px] flex-1'} ${dropzoneBase} ${
              dragging ? 'border-accent bg-accent-soft/60' : 'border-border-subtle bg-surface'
            } animate-in slide-in-from-bottom-4 fade-in duration-300 ease-out`}
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
        )
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
        backLabel={t('common.back')}
        primaryLabel={t('common.next')}
        onPrimary={onNext}
        primaryDisabled={!file || uploading}
        primaryHelper={file ? null : t('wizard.selectFileFirst')}
      />
    </div>
  );
}
