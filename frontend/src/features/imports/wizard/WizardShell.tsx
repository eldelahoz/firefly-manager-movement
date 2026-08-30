import { Fragment } from 'react';
import { DialogContent, DialogTitle } from '@/shared/ui/dialog';
import { Button } from '@/shared/ui/button';

function StepDots({ steps, current }: { steps: string[]; current: number }) {
  return (
    <div className="flex items-start" aria-hidden="true">
      {steps.map((label, i) => (
        <Fragment key={label}>
          {i > 0 && <span className="mt-[5px] h-0.5 w-10 bg-border-subtle" />}
          <div className="flex flex-col items-center gap-1">
            <span
              className={`size-3 rounded-full ${
                i < current
                  ? 'bg-accent-soft ring-1 ring-accent'
                  : i === current
                    ? 'bg-accent'
                    : 'bg-border-subtle'
              }`}
            />
            <span className="text-[10px] leading-none whitespace-nowrap text-text-2">
              {i + 1} {label}
            </span>
          </div>
        </Fragment>
      ))}
    </div>
  );
}

export function WizardShell({
  title,
  subtitle,
  steps,
  current,
  children,
}: {
  title: string;
  subtitle: string;
  steps: string[];
  current: number;
  children: React.ReactNode;
}) {
  return (
    <DialogContent
      showCloseButton={false}
      className="flex h-[min(720px,90dvh)] w-full flex-col gap-0 p-6 sm:max-w-[960px]"
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <DialogTitle className="text-lg font-semibold text-text-1">{title}</DialogTitle>
          <p className="mt-0.5 text-xs text-text-2">{subtitle}</p>
        </div>
        <StepDots steps={steps} current={current} />
      </div>
      <div className="mt-5 flex min-h-0 flex-1 flex-col overflow-y-auto">{children}</div>
    </DialogContent>
  );
}

export function WizardFooter({
  onCancel,
  cancelLabel,
  onBack,
  backDisabled,
  backLabel,
  primaryLabel,
  onPrimary,
  primaryDisabled,
  primaryHelper,
}: {
  onCancel: () => void;
  cancelLabel: string;
  onBack?: () => void;
  backDisabled?: boolean;
  backLabel?: string;
  primaryLabel: string;
  onPrimary?: () => void;
  primaryDisabled?: boolean;
  primaryHelper?: string | null;
}) {
  return (
    <div className="mt-auto flex items-center justify-between pt-4">
      <Button variant="ghost" className="text-accent" onClick={onCancel}>
        {cancelLabel}
      </Button>
      <div className="flex flex-col items-end gap-1">
        <div className="flex items-center gap-3">
          {onBack && (
            <Button variant="cancel" onClick={onBack} disabled={backDisabled}>
              {backLabel}
            </Button>
          )}
          {onPrimary && (
            <Button onClick={onPrimary} disabled={primaryDisabled}>
              {primaryLabel}
            </Button>
          )}
        </div>
        {primaryDisabled && primaryHelper && (
          <p className="text-[11px] text-text-3">{primaryHelper}</p>
        )}
      </div>
    </div>
  );
}
