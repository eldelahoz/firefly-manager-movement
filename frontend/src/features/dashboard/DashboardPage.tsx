import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useImportWizard } from '@/app/importWizardContext';
import { HealthCard } from './HealthCard';
import { KpiCards } from './KpiCards';
import { RecentBatches } from './RecentBatches';
import { BrowseWidget } from '@/features/firefly/BrowseWidget';
import { Button } from '@/shared/ui/button';
import { usePageHeader } from '@/shared/ui/PageHeader';

export function DashboardPage() {
  const { t } = useTranslation();
  const openWizard = useImportWizard();
  const actions = useMemo(
    () => <Button onClick={() => openWizard()}>{t('common.newImport')}</Button>,
    [openWizard, t],
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
