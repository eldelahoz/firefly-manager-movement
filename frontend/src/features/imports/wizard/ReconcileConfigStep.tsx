// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function ReconcileConfigStep(_props: {
  batchId: number | null;
  currency: string;
  submitting: boolean;
  error: string | null;
  onStart: () => void;
  onError: (message: string) => void;
  onDone: (batchId: number) => void;
}) {
  return <div data-testid="step-reconcile" />;
}
