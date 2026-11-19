import { useTranslation } from 'react-i18next';
import { formatSignedMoney } from '@/shared/lib/money';
import { cn } from '@/shared/lib/utils';

/** Money rule (DS): sign always (true minus U+2212), money in = matched green, money out = neutral text.
 *  Colour never carries the meaning alone: the sign does (WCAG 1.4.1). */
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
  const tone = cents > 0 ? 'text-status-matched' : 'text-text-1';
  return (
    <span className={cn('tabular-money whitespace-nowrap', tone, className)}>
      {formatSignedMoney(cents, currency, i18n.language)}
    </span>
  );
}
