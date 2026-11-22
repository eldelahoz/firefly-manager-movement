import { ArrowDownLeft, ArrowLeftRight, ArrowUpRight } from 'lucide-react';
import type { TransactionType } from './customization';

const ICONS = { withdrawal: ArrowUpRight, deposit: ArrowDownLeft, transfer: ArrowLeftRight };

/** Decorative: the type is always written next to it. */
export function TransactionTypeIcon({ type }: { type: TransactionType }) {
  const Icon = ICONS[type];
  return <Icon aria-hidden="true" className="inline size-3.5 shrink-0 align-[-0.2em]" />;
}
