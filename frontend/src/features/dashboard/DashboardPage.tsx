import { useTranslation } from 'react-i18next';
import { HealthCard } from './HealthCard';
import { KpiCards } from './KpiCards';
import { RecentBatches } from './RecentBatches';
import { BrowseWidget } from '@/features/firefly/BrowseWidget';
import { PageHeader } from '@/shared/ui/PageHeader';

export function DashboardPage() {
  const { t } = useTranslation();
  return (
    <div className="space-y-6">
      <PageHeader title={t('dashboard.title')} />
      <HealthCard />
      <KpiCards />
      <div className="grid gap-6 lg:grid-cols-2">
        <RecentBatches />
        <BrowseWidget />
      </div>
    </div>
  );
}
