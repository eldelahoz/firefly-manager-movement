import { useEffect, useState, useSyncExternalStore } from 'react';
import { toast, type ToastT } from 'sonner';
import { useTranslation } from 'react-i18next';
import { ChevronDown, ChevronUp } from 'lucide-react';
import type { Ff3Account, MovementDTO, SuggestedCounterparty } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { MoneyText } from '@/shared/ui/MoneyText';
import { Button } from '@/shared/ui/button';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/shared/ui/dialog';
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from '@/shared/ui/sheet';
import type { Customization } from './customization';
import { PersonalizeEditor } from './PersonalizeEditor';

const WIDE = '(min-width: 900px)';
const subscribe = (cb: () => void) => {
  const mq = window.matchMedia(WIDE);
  mq.addEventListener('change', cb);
  return () => mq.removeEventListener('change', cb);
};
const useWide = () => useSyncExternalStore(subscribe, () => window.matchMedia(WIDE).matches);

type Pending = 'close' | 'prev' | 'next' | null;

/** One modal sheet for editing a movement (with prev/next) or the selection (bulk, fill-only).
 *  Right side on wide screens, bottom sheet on phones. Nothing reaches Firefly III until Apply. */
export function EditSheet({
  movement,
  position,
  bulkCount,
  bulkDirection,
  initial,
  currency,
  onNavigate,
  onSave,
  onClose,
  ...lists
}: {
  /** Single mode; omitted in bulk mode. */
  movement?: MovementDTO;
  position?: { index: number; total: number };
  bulkCount?: number;
  /** Bulk only: the selection's shared direction, undefined when mixed. */
  bulkDirection?: 'withdrawal' | 'deposit';
  initial: Customization;
  /** Single mode: the backend suggestion behind `initial`, for the mark and "Clear suggestion". */
  suggestion?: SuggestedCounterparty | null;
  currency: string;
  defaultTag: string | null;
  accounts: Ff3Account[];
  batchAccountId: number | null;
  tags: string[];
  categories: string[];
  budgets: string[];
  onNavigate?: (step: -1 | 1) => void;
  onSave: (c: Customization) => void;
  onClose: () => void;
}) {
  const { t, i18n } = useTranslation();
  const wide = useWide();
  const [dirty, setDirty] = useState(false);
  const [pending, setPending] = useState<Pending>(null);
  // Closing flips `open` so Radix plays the exit and returns focus; the parent unmounts us afterwards.
  const [open, setOpen] = useState(true);
  // Opened from state (no Radix Trigger), so Radix has nowhere to return focus: remember the opener.
  const [opener] = useState(() =>
    document.activeElement instanceof HTMLElement ? document.activeElement : null,
  );

  // The modal hides toasts from keyboard and screen readers: hold toasts with an action (Undo)
  // while it is open, and give them their full time again once it closes.
  useEffect(() => {
    if (!open) return;
    const held = toast
      .getToasts()
      .filter((x): x is ToastT => 'action' in x && !!x.action && x.duration !== Infinity);
    for (const x of held) toast(x.title, { ...x, duration: Infinity });
    return () => {
      const active = new Set(toast.getToasts().map((x) => x.id));
      for (const x of held) if (active.has(x.id)) toast(x.title, x);
    };
  }, [open]);

  function run(action: Exclude<Pending, null>) {
    setPending(null);
    setDirty(false);
    if (action === 'close') setOpen(false);
    else onNavigate?.(action === 'prev' ? -1 : 1);
  }
  const request = (action: Exclude<Pending, null>) => (dirty ? setPending(action) : run(action));

  return (
    <Sheet open={open} onOpenChange={(o) => !o && request('close')}>
      <SheetContent
        // A toast (Undo) lives outside the sheet; clicking it must not close the sheet.
        onInteractOutside={(e) => {
          if ((e.target as Element | null)?.closest('[data-sonner-toaster]')) e.preventDefault();
        }}
        onCloseAutoFocus={(e) => {
          e.preventDefault();
          (opener?.isConnected
            ? opener
            : document.getElementById('results-missing-table')
          )?.focus();
          onClose();
        }}
        side={wide ? 'right' : 'bottom'}
        className={`ffm-sheet gap-0 ${wide ? 'w-full sm:max-w-[440px]' : 'max-h-[85dvh] rounded-t-xl'}`}
      >
        <SheetHeader>
          <div className="flex items-center gap-2 pr-8">
            <SheetTitle className="text-lg">
              {movement ? t('results.editTitle') : t('results.bulkTitle', { count: bulkCount })}
            </SheetTitle>
            {movement && position && (
              <div className="ml-auto flex gap-1">
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label={t('results.prevMovement')}
                  title={t('results.prevMovement')}
                  disabled={position.index === 0}
                  onClick={() => request('prev')}
                >
                  <ChevronUp className="size-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  aria-label={t('results.nextMovement')}
                  title={t('results.nextMovement')}
                  disabled={position.index === position.total - 1}
                  onClick={() => request('next')}
                >
                  <ChevronDown className="size-4" />
                </Button>
              </div>
            )}
          </div>
          <SheetDescription>
            {movement && position
              ? t('results.position', { index: position.index + 1, total: position.total })
              : t('results.bulkHint')}
          </SheetDescription>
          {movement && (
            <div className="mt-2 grid gap-1 rounded-lg border border-border-subtle bg-canvas p-3 text-sm">
              <div className="flex items-baseline justify-between gap-3">
                <span className="min-w-0 break-words font-medium">{movement.description}</span>
                <MoneyText cents={movement.amount_cents} currency={currency} />
              </div>
              <span className="text-xs text-text-2 tabular-nums">
                {formatDate(movement.date, i18n.language)} ·{' '}
                {t('results.line', { line: movement.line_no })}
              </span>
            </div>
          )}
        </SheetHeader>
        <PersonalizeEditor
          // A new key per movement resets the form on prev/next.
          key={movement?.id ?? 'bulk'}
          mode={movement ? 'single' : 'bulk'}
          initial={initial}
          statementDescription={movement?.description}
          direction={
            movement ? (movement.amount_cents < 0 ? 'withdrawal' : 'deposit') : bulkDirection
          }
          {...lists}
          currency={currency}
          onDirtyChange={setDirty}
          onSave={(c) => {
            onSave(c);
            run('close');
          }}
          onCancel={() => request('close')}
          submitLabel={movement ? t('results.save') : t('results.applyToSelection')}
        />
      </SheetContent>
      {pending && (
        <Dialog open onOpenChange={(o) => !o && setPending(null)}>
          <DialogContent className="sm:max-w-sm">
            <DialogHeader>
              <DialogTitle>{t('results.discardTitle')}</DialogTitle>
              <p className="text-sm text-text-2">{t('results.discardBody')}</p>
            </DialogHeader>
            <div className="flex justify-end gap-2">
              <Button variant="cancel" onClick={() => setPending(null)}>
                {t('results.keepEditing')}
              </Button>
              <Button onClick={() => run(pending)}>{t('results.discard')}</Button>
            </div>
          </DialogContent>
        </Dialog>
      )}
    </Sheet>
  );
}
