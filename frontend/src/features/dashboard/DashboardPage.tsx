import { useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { HealthCard } from './HealthCard';
import { KpiCards } from './KpiCards';
import { RecentBatches } from './RecentBatches';
import { BrowseWidget } from '@/features/firefly/BrowseWidget';
import { Button } from '@/shared/ui/button';
import { usePageHeader } from '@/shared/ui/PageHeader';

export function DashboardPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const actions = useMemo(
    () => <Button onClick={() => navigate('/imports')}>{t('common.newImport')}</Button>,
    [navigate, t],
  );
  usePageHeader({ title: t('dashboard.title'), actions });
  return (
    <div className="space-y-6">
      <HealthCard />
      <KpiCards />
      <div className="grid gap-6 lg:grid-cols-2">
        <RecentBatches />
        <BrowseWidget />
      </div>
    </div>
  );
}
