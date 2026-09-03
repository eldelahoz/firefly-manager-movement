import type { Ff3Account } from '@/shared/api/dto';

export interface OriginSuggestion {
  account: Ff3Account | null;
  warning: boolean;
  values: string[];
}

const normIban = (v: string) => v.replace(/\s+/g, '').toUpperCase();
const norm = (v: string) => v.trim().toLowerCase();

function matches(value: string, a: Ff3Account): boolean {
  return (
    (a.iban ? normIban(a.iban) === normIban(value) : false) ||
    (a.account_number ? norm(a.account_number) === norm(value) : false) ||
    norm(a.name) === norm(value)
  );
}

export function suggestAccount(values: string[], accounts: Ff3Account[]): OriginSuggestion {
  if (values.length === 0) return { account: null, warning: false, values };
  const matched = accounts.filter((a) => values.some((v) => matches(v, a)));
  if (values.length === 1 && matched.length === 1) {
    return { account: matched[0], warning: false, values };
  }
  return { account: null, warning: true, values };
}
