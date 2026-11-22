import type { Ff3Account } from '@/shared/api/dto';

/** RF-08 v0.10: an existing account, a new expense/revenue account by name, or null/undefined = Cash.
 *  One field, so a bulk fill replaces the whole choice (null = explicit Cash over each row). */
export type Counterparty = { id: number } | { name: string } | null;

export interface Customization {
  counterparty?: Counterparty;
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
    'counterparty' | 'description' | 'category' | 'budget' | 'tags' | 'notes' | 'attachment'
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

export type Direction = 'withdrawal' | 'deposit';
export type TransactionType = Direction | 'transfer';

/** Mirror of the backend rule, for display only: an own asset is a transfer, anything else follows the sign.
 *  Undefined when the account is not in the loaded lists: the type is not guessed. */
export function transactionType(
  direction: Direction,
  counterparty: Counterparty | undefined,
  accounts: Ff3Account[],
): TransactionType | undefined {
  if (!counterparty || !('id' in counterparty)) return direction;
  const account = accounts.find((a) => a.id === counterparty.id);
  if (!account) return undefined;
  return account.type === 'asset' ? 'transfer' : direction;
}
