import { useTranslation } from 'react-i18next';
import { moneyParts } from '@/shared/lib/money';
import { useNumberFormat } from '@/shared/lib/numberFormat';
import { cn } from '@/shared/lib/utils';

/** Money rule (DS): sign always (true minus U+2212), money in = matched green, money out = neutral text.
 *  Colour never carries the meaning alone: the sign does (WCAG 1.4.1).
 *  The decimal separator and cents are muted so they never read as a thousands group (NFR-08). */
export function MoneyText({
  cents,
  currency,
  className,
}: {
  cents: number;
  currency: string;
  className?: string;
}) {
  const { i18n } = useTranslation();
  const p = moneyParts(cents, currency, i18n.resolvedLanguage ?? i18n.language, useNumberFormat());
  const tone = cents > 0 ? 'text-status-matched' : 'text-text-1';
  return (
    <span className={cn('tabular-money whitespace-nowrap', tone, className)}>
      {p.sign}
      <span data-part="integer">{p.integer}</span>
      <span data-part="fraction" className="text-text-2">
        {p.fraction}
      </span>
      {` ${p.unit}`}
    </span>
  );
}
