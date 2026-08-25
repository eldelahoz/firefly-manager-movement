import { useTranslation } from 'react-i18next';
import { HealthCard } from './HealthCard';
import { KpiCards } from './KpiCards';
import { RecentBatches } from './RecentBatches';
import { BrowseWidget } from '@/features/firefly/BrowseWidget';

export function DashboardPage() {
  const { t } = useTranslation();
  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">{t('dashboard.title')}</h1>
      <HealthCard />
      <KpiCards />
      <div className="grid gap-6 lg:grid-cols-2">
        <RecentBatches />
        <BrowseWidget />
      </div>
    </div>
  );
}
