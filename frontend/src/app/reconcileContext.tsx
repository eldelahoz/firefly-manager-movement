import { createContext, useCallback, useContext, useState } from 'react';
import { ReconcileDialog } from '@/features/imports/wizard/ReconcileDialog';

const ReconcileContext = createContext<(batchId: number) => void>(() => {});
export const useRerun = () => useContext(ReconcileContext);

export function ReconcileProvider({ children }: { children: React.ReactNode }) {
  const [rerunBatchId, setRerunBatchId] = useState<number | null>(null);
  const open = useCallback((batchId: number) => setRerunBatchId(batchId), []);
  return (
    <ReconcileContext.Provider value={open}>
      {children}
      <ReconcileDialog batchId={rerunBatchId} onClose={() => setRerunBatchId(null)} />
    </ReconcileContext.Provider>
  );
}
