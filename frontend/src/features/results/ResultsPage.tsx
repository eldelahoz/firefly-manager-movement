import { useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getImport, getResults } from '@/shared/api/endpoints';
import { formatDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { EmptyState } from '@/shared/ui/EmptyState';
import { Button } from '@/shared/ui/button';
import { Checkbox } from '@/shared/ui/checkbox';
import { MoneyText } from '@/shared/ui/MoneyText';
import { PageHeader } from '@/shared/ui/PageHeader';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui/tabs';
import type { ApplyReport } from '@/shared/api/dto';
import { useRerun } from '@/app/reconcileContext';
import { VirtualList } from './VirtualList';
import { MissingRow } from './MissingRow';
import { ExtraRow } from './ExtraRow';
import { ApplyPanel } from './ApplyPanel';

type Tab = 'matched' | 'missing' | 'extra';

export function ResultsPage() {
  const { batchId } = useParams();
  const id = Number(batchId);
  const { t, i18n } = useTranslation();
  const [tab, setTab] = useState<Tab>('matched');
  const [selected, setSelected] = useState<number[]>([]);
  const [panelOpen, setPanelOpen] = useState(false);
  const [reports, setReports] = useState<{ batchId: number; items: ApplyReport[] }>({
    batchId: NaN,
    items: [],
  });
  const openRerun = useRerun();
  const detail = useQuery({ queryKey: queryKeys.import(id), queryFn: () => getImport(id) });
  const results = useQuery({
    queryKey: queryKeys.results(id),
    queryFn: () => getResults(id),
    retry: false,
  });
  if (!Number.isFinite(id)) {
    return (
      <EmptyState
        title={t('results.noReconciliation')}
        action={
          <Button asChild>
            <Link to="/imports">{t('results.startImport')}</Link>
          </Button>
        }
      />
    );
  }
  if (results.isPending || detail.isPending) return <TableSkeleton rows={6} cols={4} />;
  if (results.isError || detail.isError) {
    return (
      <EmptyState
        title={t('results.noReconciliation')}
        action={
          <Button asChild>
            <Link to="/imports">{t('results.startImport')}</Link>
          </Button>
        }
      />
    );
  }
  const counts = {
    matched: results.data.matched.length,
    missing: results.data.missing.length,
    extra: results.data.extra.length,
  };
  const activeSelected = selected.filter((sid) => results.data.missing.some((m) => m.id === sid));
  const currency = detail.data?.batch.currency ?? 'EUR';
  const tabs: Tab[] = ['matched', 'missing', 'extra'];
  return (
    <div className="space-y-4">
      <PageHeader
        title={
          <span className="truncate">
            {t('results.title')} — {detail.data?.batch.filename ?? `#${id}`}
          </span>
        }
        actions={
          <Button variant="outline" onClick={() => openRerun(id)}>
            {t('imports.rerun')}
          </Button>
        }
      />
      <Tabs value={tab} onValueChange={(v) => setTab(v as Tab)} className="gap-4">
        <TabsList
          variant="line"
          className="sticky top-0 z-[5] w-full justify-start rounded-none border-b border-border-subtle bg-canvas py-1"
        >
          {tabs.map((tb) => (
            <TabsTrigger key={tb} value={tb}>
              {t(`results.${tb}`)} ({counts[tb]})
            </TabsTrigger>
          ))}
        </TabsList>
        <TabsContent value="matched">
          {counts.matched === 0 ? (
            <EmptyState title={t('results.matched')} />
          ) : (
            <VirtualList
              count={counts.matched}
              renderItem={(i) => {
                const { movement, ff3 } = results.data.matched[i];
                return (
                  <div className="grid grid-cols-[1fr_1fr] gap-2 border-b border-border-subtle py-3 text-sm">
                    <div className="min-w-0">
                      <p className="truncate">{movement.description}</p>
                      <p className="text-xs text-text-3">
                        #{movement.line_no} · {formatDate(movement.date, i18n.language)}
                      </p>
                      <MoneyText cents={movement.amount_cents} currency={currency} />
                    </div>
                    <div className="min-w-0">
                      <p className="truncate">{ff3.description}</p>
                      <p className="text-xs text-text-3">
                        FF3 {ff3.ff3_id} · {formatDate(ff3.date, i18n.language)}
                      </p>
                      <MoneyText cents={ff3.amount_cents} currency={currency} />
                    </div>
                  </div>
                );
              }}
            />
          )}
        </TabsContent>
        <TabsContent value="missing">
          {counts.missing === 0 ? (
            <EmptyState title={t('results.missing')} />
          ) : (
            <>
              <div className="flex items-center justify-between py-2">
                <label className="flex items-center gap-2 text-sm text-text-2">
                  <Checkbox
                    aria-label={t('results.selectAll')}
                    checked={activeSelected.length === counts.missing && counts.missing > 0}
                    onCheckedChange={() =>
                      setSelected(
                        activeSelected.length === counts.missing
                          ? []
                          : results.data.missing.map((m) => m.id),
                      )
                    }
                  />
                  {t('results.selectAll')}
                </label>
                <Button onClick={() => setPanelOpen(true)} disabled={activeSelected.length === 0}>
                  {t('results.apply')} ({activeSelected.length})
                </Button>
              </div>
              <VirtualList
                count={counts.missing}
                renderItem={(i) => (
                  <MissingRow
                    movement={results.data.missing[i]}
                    currency={currency}
                    batchId={id}
                    selected={activeSelected.includes(results.data.missing[i].id)}
                    report={
                      reports.batchId === id
                        ? reports.items.find((r) => r.movement_id === results.data.missing[i].id)
                        : undefined
                    }
                    onToggle={(mid) =>
                      setSelected((s) =>
                        s.includes(mid) ? s.filter((x) => x !== mid) : [...s, mid],
                      )
                    }
                  />
                )}
              />
            </>
          )}
        </TabsContent>
        <TabsContent value="extra">
          {counts.extra === 0 ? (
            <EmptyState title={t('results.extra')} />
          ) : (
            <VirtualList
              count={counts.extra}
              renderItem={(i) => <ExtraRow tx={results.data.extra[i]} currency={currency} />}
            />
          )}
        </TabsContent>
      </Tabs>
      {panelOpen && tab === 'missing' && (
        <ApplyPanel
          batchId={id}
          selected={activeSelected}
          onClose={() => setPanelOpen(false)}
          onDone={(rs) => {
            setReports({ batchId: id, items: rs });
            setSelected([]);
            setPanelOpen(false);
          }}
        />
      )}
    </div>
  );
}
