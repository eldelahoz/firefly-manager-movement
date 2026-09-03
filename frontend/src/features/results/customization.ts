export interface Customization {
  counterpartyId?: number;
  description?: string;
  category?: string;
  budget?: string;
  tags?: string[];
  notes?: string;
  attachment?: { filename: string; file: File };
}

export type BulkFields = Partial<
  Pick<
    Customization,
    'counterpartyId' | 'description' | 'category' | 'budget' | 'tags' | 'notes' | 'attachment'
  >
>;

/** Fill-only: only keys present (non-undefined) in `bulk` overwrite each movement's customization. */
export function applyBulkFillOnly(
  current: Record<number, Customization>,
  ids: number[],
  bulk: BulkFields,
): Record<number, Customization> {
  const next = { ...current };
  for (const id of ids) {
    const existing = next[id] ?? {};
    const applied: Customization = { ...existing };
    (Object.keys(bulk) as (keyof BulkFields)[]).forEach((k) => {
      if (bulk[k] !== undefined) (applied as Record<string, unknown>)[k] = bulk[k];
    });
    next[id] = applied;
  }
  return next;
}
