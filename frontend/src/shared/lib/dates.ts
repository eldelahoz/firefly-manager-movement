export function isValidIsoDate(s: string): boolean {
  return /^\d{4}-\d{2}-\d{2}$/.test(s) && !Number.isNaN(Date.parse(`${s}T00:00:00Z`));
}

export function toIsoDate(d: Date): string {
  if (Number.isNaN(d.getTime())) throw new RangeError('Invalid time value');
  return d.toISOString().slice(0, 10);
}

export function addDays(iso: string, days: number): string | undefined {
  if (!isValidIsoDate(iso)) return undefined;
  const d = new Date(`${iso}T00:00:00Z`);
  d.setUTCDate(d.getUTCDate() + days);
  return toIsoDate(d);
}

export function formatDate(iso: string, locale: string): string {
  return new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeZone: 'UTC' }).format(
    new Date(`${iso}T00:00:00Z`),
  );
}
