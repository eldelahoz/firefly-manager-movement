import { useTranslation } from 'react-i18next';
import { formatMoney, formatSignedMoney } from '@/shared/lib/money';
import { cn } from '@/shared/lib/utils';

export function MoneyText({
  cents,
  currency,
  signed = false,
  colored = false,
  className,
}: {
  cents: number;
  currency: string;
  signed?: boolean;
  colored?: boolean;
  className?: string;
}) {
  const { i18n } = useTranslation();
  const text = signed
    ? formatSignedMoney(cents, currency, i18n.language)
    : formatMoney(cents, currency, i18n.language);
  const tone = colored
    ? cents < 0
      ? 'text-status-extra-txt'
      : 'text-status-matched-txt'
    : 'text-text-1';
  return <span className={cn('tabular-money', tone, className)}>{text}</span>;
}
