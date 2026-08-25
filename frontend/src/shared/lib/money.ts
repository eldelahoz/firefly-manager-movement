export function formatMoney(cents: number, currency: string, locale: string): string {
  return new Intl.NumberFormat(locale, { style: 'currency', currency }).format(cents / 100);
}

export function formatSignedMoney(cents: number, currency: string, locale: string): string {
  if (cents === 0) return formatMoney(cents, currency, locale);
  return cents > 0
    ? `+${formatMoney(cents, currency, locale)}`
    : `−${formatMoney(Math.abs(cents), currency, locale)}`;
}
