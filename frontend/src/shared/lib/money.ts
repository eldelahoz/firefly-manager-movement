export function formatMoney(cents: number, currency: string, locale: string): string {
  const parts = new Intl.NumberFormat(locale, { style: 'currency', currency }).formatToParts(
    cents / 100,
  );
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
