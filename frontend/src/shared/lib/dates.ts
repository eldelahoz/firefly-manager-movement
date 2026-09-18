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

export const DATE_FORMATS = [
  { value: 'dd/mm/yyyy', label: 'DD/MM/YYYY' },
  { value: 'mm/dd/yyyy', label: 'MM/DD/YYYY' },
  { value: 'yyyy-mm-dd', label: 'YYYY-MM-DD' },
  { value: 'dd.mm.yyyy', label: 'DD.MM.YYYY' },
  { value: 'dd-mm-yyyy', label: 'DD-MM-YYYY' },
  { value: 'yyyy/mm/dd', label: 'YYYY/MM/DD' },
] as const;

export function parseWithFormat(raw: string, format?: string): string | null {
  const s = raw.trim();
  if (!s) return null;
  if (!format || format === 'auto') {
    for (const f of [
      'yyyy-mm-dd',
      'dd/mm/yyyy',
      'dd.mm.yyyy',
      'dd-mm-yyyy',
      'mm/dd/yyyy',
      'yyyy/mm/dd',
    ]) {
      const d = tryParse(s, f);
      if (d) return d;
    }
    return null;
  }
  return tryParse(s, format);
}

export function isAmbiguousFor(raw: string, format?: string): boolean {
  if (format && format !== 'auto' && format !== 'dd/mm/yyyy' && format !== 'mm/dd/yyyy') {
    return false;
  }
  const m = raw.trim().match(/^(\d{1,2})[/.\-](\d{1,2})[/.\-](\d{4})$/);
  if (!m) return false;
  return Number(m[1]) <= 12 && Number(m[2]) <= 12;
}

function tryParse(s: string, format: string): string | null {
  const sep = format.includes('/') ? '/' : format.includes('.') ? '.' : '-';
  const parts = s.split(sep);
  if (parts.length !== 3) return null;
  const order = format.split(sep);
  const get = (p: string) => parts[order.indexOf(p)];
  const y = Number(get('yyyy'));
  const mo = Number(get('mm'));
  const d = Number(get('dd'));
  if (!y || !mo || !d || mo > 12 || d > 31) return null;
  const dt = new Date(Date.UTC(y, mo - 1, d));
  if (dt.getUTCFullYear() !== y || dt.getUTCMonth() !== mo - 1 || dt.getUTCDate() !== d) {
    return null;
  }
  return `${y}-${String(mo).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
}
