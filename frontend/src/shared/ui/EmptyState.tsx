import type { ReactNode } from 'react';

export function EmptyState({ title, action }: { title: string; action?: ReactNode }) {
  return (
    <div className="flex flex-col items-center gap-4 rounded-xl border border-dashed border-border-strong bg-surface p-12 text-center">
      <p className="text-text-2">{title}</p>
      {action}
    </div>
  );
}
