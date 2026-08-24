export const queryKeys = {
  health: ['health'] as const,
  accounts: ['accounts'] as const,
  imports: ['imports'] as const,
  import: (id: number) => ['import', id] as const,
  results: (batchId: number) => ['results', batchId] as const,
  accountTxs: (id: string, start: string, end: string) => ['account-txs', id, start, end] as const,
};
