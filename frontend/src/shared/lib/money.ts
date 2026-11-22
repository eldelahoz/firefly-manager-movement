export function formatMoney(cents: number, currency: string, locale: string): string {
  // Amounts are always cents: CLDR's 0 decimals for COP etc. would round them away.
  const parts = new Intl.NumberFormat(locale, {
    style: 'currency',
    currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).formatToParts(cents / 100);
  const unit = parts.find((p) => p.type === 'currency')?.value ?? currency;
  const amount = parts
    .filter((p) => p.type !== 'currency' && p.type !== 'literal')
    .map((p) => p.value)
    .join('');
  return `${amount} ${unit}`;
}

export function formatSignedMoney(cents: number, currency: string, locale: string): string {
  if (cents === 0) return formatMoney(cents, currency, locale);
  return cents > 0
    ? `+${formatMoney(cents, currency, locale)}`
    : `−${formatMoney(Math.abs(cents), currency, locale)}`;
}
