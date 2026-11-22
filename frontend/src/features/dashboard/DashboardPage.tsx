import { useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { useImportWizard } from '@/app/importWizardContext';
import { getImports } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';
import { HealthCard } from './HealthCard';
import { NeedsYou } from './NeedsYou';
import { RecentBatches } from './RecentBatches';
import { BrowseWidget } from '@/features/firefly/BrowseWidget';
import { Button } from '@/shared/ui/button';
import { EmptyState } from '@/shared/ui/EmptyState';
import { usePageHeader } from '@/shared/ui/PageHeader';
import { Skeleton } from '@/shared/ui/Skeleton';

export function DashboardPage() {
  const { t } = useTranslation();
  const openWizard = useImportWizard();
  const imports = useQuery({ queryKey: queryKeys.imports, queryFn: getImports });
  const actions = useMemo(
    () => <Button onClick={() => openWizard()}>{t('common.newImport')}</Button>,
    [openWizard, t],
  );
  usePageHeader({ title: t('dashboard.title'), actions });
  const batches = imports.data ?? [];
  return (
    <div className="space-y-6">
      <HealthCard />
      {imports.isPending ? (
        <Skeleton className="h-40 w-full rounded-xl" />
      ) : imports.isError && batches.length === 0 ? (
        <div role="alert">
          <EmptyState
            title={t('common.error')}
            action={
              <Button
                variant="outline"
                disabled={imports.isFetching}
                onClick={() => void imports.refetch()}
              >
                {t('common.retry')}
              </Button>
            }
          />
        </div>
      ) : batches.length === 0 ? (
        <EmptyState
          title={t('dashboard.firstRun')}
          action={<Button onClick={() => openWizard()}>{t('common.newImport')}</Button>}
        />
      ) : (
        <>
          <NeedsYou batches={batches} />
          <RecentBatches batches={batches} />
        </>
      )}
      <BrowseWidget />
    </div>
  );
}
