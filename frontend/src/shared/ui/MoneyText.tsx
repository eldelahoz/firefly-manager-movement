import { useTranslation } from 'react-i18next';
import { formatMoney, formatSignedMoney } from '@/shared/lib/money';

export function MoneyText({
  cents,
  currency,
  signed = false,
  colored = false,
}: {
  cents: number;
  currency: string;
  signed?: boolean;
  colored?: boolean;
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
  return <span className={`tabular-money ${tone}`}>{text}</span>;
}
