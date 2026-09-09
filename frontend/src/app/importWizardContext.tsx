import { createContext, useCallback, useContext, useState } from 'react';
import { ImportWizard } from '@/features/imports/wizard/ImportWizard';

type OpenWizard = (resumeBatchId?: number | null) => void;

const ImportWizardContext = createContext<OpenWizard>(() => {});
export const useImportWizard = () => useContext(ImportWizardContext);

export function ImportWizardProvider({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  const [resumeBatchId, setResumeBatchId] = useState<number | null>(null);
  const [wizardKey, setWizardKey] = useState(0);

  const openWizard = useCallback((batchId?: number | null) => {
    setResumeBatchId(batchId ?? null);
    setWizardKey((k) => k + 1);
    setOpen(true);
  }, []);

  const closeWizard = useCallback(() => {
    setOpen(false);
    setResumeBatchId(null);
  }, []);

  return (
    <ImportWizardContext.Provider value={openWizard}>
      {children}
      <ImportWizard
        key={wizardKey}
        open={open}
        onClose={closeWizard}
        resumeBatchId={resumeBatchId}
      />
    </ImportWizardContext.Provider>
  );
}
