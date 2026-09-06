import { type ReactNode } from 'react';
import { cn } from '@/shared/lib/utils';

export function Card({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <section
      className={cn('rounded-xl border border-border-subtle bg-surface p-4 shadow-sm', className)}
    >
      {children}
    </section>
  );
}

export function CardHeader({ children, className }: { children: ReactNode; className?: string }) {
  return <h2 className={cn('font-semibold', className)}>{children}</h2>;
}

export function CardContent({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn('mt-3 space-y-3', className)}>{children}</div>;
}

export function CardFooter({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn('flex gap-2 pt-2', className)}>{children}</div>;
}
