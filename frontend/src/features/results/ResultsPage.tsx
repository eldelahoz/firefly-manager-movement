import { useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { getImport, getResults } from '@/shared/api/endpoints';
import { formatDate } from '@/shared/lib/dates';
import { queryKeys } from '@/shared/lib/keys';
import { EmptyState } from '@/shared/ui/EmptyState';
import { MoneyText } from '@/shared/ui/MoneyText';
import { TableSkeleton } from '@/shared/ui/Skeleton';
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
          <Link to="/imports" className="rounded-lg bg-accent px-4 py-2 text-white">
            {t('results.startImport')}
          </Link>
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
          <Link to="/imports" className="rounded-lg bg-accent px-4 py-2 text-white">
            {t('results.startImport')}
          </Link>
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
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h1 className="truncate text-2xl font-bold">
          {t('results.title')} — {detail.data?.batch.filename ?? `#${id}`}
        </h1>
        <button
          onClick={() => openRerun(id)}
          className="h-11 rounded-lg border border-border-subtle bg-surface px-4 text-sm font-medium text-text-2"
        >
          {t('imports.rerun')}
        </button>
      </div>
      <div
        role="tablist"
        className="sticky top-0 z-[5] flex gap-1 border-b border-border-subtle bg-canvas py-1"
      >
        {tabs.map((tb) => (
          <button
            key={tb}
            role="tab"
            aria-selected={tab === tb}
            onClick={() => setTab(tb)}
            className={`rounded-t-lg px-4 py-2 text-sm font-medium ${tab === tb ? 'border-b-2 border-accent text-accent' : 'text-text-2'}`}
          >
            {t(`results.${tb}`)} ({counts[tb]})
          </button>
        ))}
      </div>
      <div role="tabpanel">
        {tab === 'matched' &&
          (counts.matched === 0 ? (
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
          ))}
        {tab === 'missing' &&
          (counts.missing === 0 ? (
            <EmptyState title={t('results.missing')} />
          ) : (
            <>
              <div className="flex items-center justify-between py-2">
                <label className="flex items-center gap-2 text-sm text-text-2">
                  <input
                    type="checkbox"
                    checked={activeSelected.length === counts.missing && counts.missing > 0}
                    onChange={() =>
                      setSelected(
                        activeSelected.length === counts.missing
                          ? []
                          : results.data.missing.map((m) => m.id),
                      )
                    }
                    className="size-5"
                  />
                  {t('results.selectAll')}
                </label>
                <button
                  onClick={() => setPanelOpen(true)}
                  disabled={activeSelected.length === 0}
                  className="h-11 rounded-lg bg-accent px-4 font-medium text-white disabled:opacity-50"
                >
                  {t('results.apply')} ({activeSelected.length})
                </button>
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
          ))}
        {tab === 'extra' &&
          (counts.extra === 0 ? (
            <EmptyState title={t('results.extra')} />
          ) : (
            <VirtualList
              count={counts.extra}
              renderItem={(i) => <ExtraRow tx={results.data.extra[i]} currency={currency} />}
            />
          ))}
      </div>
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
