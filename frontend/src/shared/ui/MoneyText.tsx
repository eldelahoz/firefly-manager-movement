import { useTranslation } from 'react-i18next';
import { formatMoney, formatSignedMoney } from '@/shared/lib/money';

export function MoneyText({
  cents,
  currency,
  signed = false,
}: {
  cents: number;
  currency: string;
  signed?: boolean;
}) {
  const { i18n } = useTranslation();
  const text = signed
    ? formatSignedMoney(cents, currency, i18n.language)
    : formatMoney(cents, currency, i18n.language);
  return (
    <span className={`tabular-money ${cents < 0 ? 'text-status-extra-txt' : 'text-text-1'}`}>
      {text}
    </span>
  );
}
