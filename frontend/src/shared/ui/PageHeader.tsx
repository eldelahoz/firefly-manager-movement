import type { ReactNode } from 'react';
import { createContext, useContext, useEffect, useState } from 'react';

export type PageHeaderConfig = {
  title: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
};

const SetHeaderContext = createContext<((config: PageHeaderConfig | null) => void) | null>(null);

export function PageHeader({ title, description, actions }: PageHeaderConfig) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-2">
      <div className="min-w-0">
        <h1 className="text-2xl font-bold">{title}</h1>
        {description ? <p className="mt-1 text-text-2">{description}</p> : null}
      </div>
      {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
    </div>
  );
}

export function PageHeaderProvider({ children }: { children: ReactNode }) {
  const [header, setHeader] = useState<PageHeaderConfig | null>(null);
  return (
    <SetHeaderContext.Provider value={setHeader}>
      {header ? (
        <header className="sticky top-0 z-20 border-b border-border-subtle bg-surface px-4 py-3 shadow-sm">
          <PageHeader {...header} />
        </header>
      ) : null}
      {children}
    </SetHeaderContext.Provider>
  );
}

export function usePageHeader(config: PageHeaderConfig | null) {
  const setHeader = useContext(SetHeaderContext);
  if (!setHeader) throw new Error('usePageHeader must be used within PageHeaderProvider');
  useEffect(() => {
    setHeader(config);
    return () => setHeader(null);
    // Intentionally depend on the memoized fields, not `config` itself, to avoid a render loop.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [setHeader, config?.title, config?.description, config?.actions]);
}
