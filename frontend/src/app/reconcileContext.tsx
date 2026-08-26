import { createContext, useContext } from 'react';

export const ReconcileContext = createContext<(batchId: number) => void>(() => {});
export const useRerun = () => useContext(ReconcileContext);
