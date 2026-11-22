/** NFR-08 v0.1: each explicit number format is written as its representative Intl locale. */
export const NUMBER_FORMATS = {
  'comma-dot': 'en-US',
  'dot-comma': 'de-DE',
  'space-comma': 'fr-FR',
  'apostrophe-dot': 'de-CH',
  'comma-dot-in': 'en-IN',
} as const;

export type NumberFormat = 'auto' | keyof typeof NUMBER_FORMATS;

export function isNumberFormat(v: unknown): v is NumberFormat {
  return v === 'auto' || Object.hasOwn(NUMBER_FORMATS, v as string);
}

/** Signed amount split for display: `${sign}${integer}${fraction} ${unit}`; fraction includes the decimal separator. */
export function moneyParts(
  cents: number,
  currency: string,
  language: string,
  format: NumberFormat = 'auto',
) {
  const locale = format === 'auto' ? language : NUMBER_FORMATS[format];
  // Amounts are always cents: CLDR's 0 decimals for COP etc. would round them away.
  const options = {
    style: 'currency',
    currency,
    useGrouping: 'always',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  } as const;
  const parts = new Intl.NumberFormat(locale, options).formatToParts(Math.abs(cents) / 100);
  const join = (types: Intl.NumberFormatPartTypes[]) =>
    parts
      .filter((p) => types.includes(p.type))
      // ICU has shipped both U+2019 and U+0027 for de-CH; pin the plain apostrophe the spec shows.
      .map((p) => (p.type === 'group' && format === 'apostrophe-dot' ? "'" : p.value))
      .join('');
  // The currency unit keeps following the UI language, as before the number format existed.
  const unitParts =
    locale === language ? parts : new Intl.NumberFormat(language, options).formatToParts(0);
  return {
    sign: cents > 0 ? '+' : cents < 0 ? '−' : '',
    integer: join(['integer', 'group']),
    fraction: join(['decimal', 'fraction']),
    unit: unitParts.find((p) => p.type === 'currency')?.value ?? currency,
  };
}

/** Unsigned amount (absolute value) with its currency unit. */
export function formatMoney(
  cents: number,
  currency: string,
  language: string,
  format: NumberFormat = 'auto',
): string {
  const p = moneyParts(cents, currency, language, format);
  return `${p.integer}${p.fraction} ${p.unit}`;
}
