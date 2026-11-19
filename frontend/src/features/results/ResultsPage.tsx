import { useMemo, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import {
  getAccounts,
  getAccountsByType,
  getImport,
  getResults,
  getSettings,
  postApply,
  postIgnore,
} from '@/shared/api/endpoints';
import type { ApplyMovementRequest, ApplyReport } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
import { moveWithTransition } from '@/shared/lib/viewTransition';
import { queryKeys } from '@/shared/lib/keys';
import { EmptyState } from '@/shared/ui/EmptyState';
import { Button } from '@/shared/ui/button';
import { Checkbox } from '@/shared/ui/checkbox';
import { MoneyText } from '@/shared/ui/MoneyText';
import { StatusPill } from '@/shared/ui/StatusPill';
import { usePageHeader } from '@/shared/ui/PageHeader';
import { TableSkeleton } from '@/shared/ui/Skeleton';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui/tabs';
import { useRerun } from '@/app/reconcileContext';
import type { BulkFields, Customization } from './customization';
import { applyBulkFillOnly } from './customization';
import { VirtualList } from './VirtualList';
import { MissingRow } from './MissingRow';
import { IgnoredRow } from './IgnoredRow';
import { ExtraRow } from './ExtraRow';
import { BulkPersonalizeDialog } from './BulkPersonalizeDialog';
import { EditMovementDialog } from './EditMovementDialog';
import { ConfirmCreateDialog } from './ConfirmCreateDialog';
import { TableHeader, CELL, GRID_COLS, GROUP_ACCENT, TABLE_WRAPPER } from './tableLayout';

type Tab = 'matched' | 'missing' | 'extra';

const TAB_ACCENT: Record<Tab, string> = {
  matched:
    'data-[state=active]:text-status-matched-txt data-[state=active]:after:bg-status-matched',
  missing:
    'data-[state=active]:text-status-missing-txt data-[state=active]:after:bg-status-missing',
  extra: 'data-[state=active]:text-status-extra-txt data-[state=active]:after:bg-status-extra',
};

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      resolve(result.replace(/^data:.*?;base64,/, ''));
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

export function ResultsPage() {
  const { batchId } = useParams();
  const id = Number(batchId);
  const { t, i18n } = useTranslation();
  const qc = useQueryClient();
  const [tab, setTab] = useState<Tab>('matched');
  const [selected, setSelected] = useState<number[]>([]);
  const [customizations, setCustomizations] = useState<Record<number, Customization>>({});
  const [editingId, setEditingId] = useState<number | null>(null);
  const [bulkOpen, setBulkOpen] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
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
  const settings = useQuery({ queryKey: queryKeys.settings, queryFn: getSettings });
  const accounts = useQuery({
    queryKey: queryKeys.accounts,
    queryFn: getAccounts,
    staleTime: 300_000,
  });
  const revenueAccounts = useQuery({
    queryKey: queryKeys.accountsByType('revenue'),
    queryFn: () => getAccountsByType('revenue'),
    staleTime: 300_000,
  });
  const expenseAccounts = useQuery({
    queryKey: queryKeys.accountsByType('expense'),
    queryFn: () => getAccountsByType('expense'),
    staleTime: 300_000,
  });
  const tags = useQuery({
    queryKey: queryKeys.tags,
    queryFn: () => import('@/shared/api/endpoints').then((m) => m.getTags()),
    staleTime: 300_000,
  });
  const categories = useQuery({
    queryKey: queryKeys.categories,
    queryFn: () => import('@/shared/api/endpoints').then((m) => m.getCategories()),
    staleTime: 300_000,
  });
  const budgets = useQuery({
    queryKey: queryKeys.budgets,
    queryFn: () => import('@/shared/api/endpoints').then((m) => m.getBudgets()),
    staleTime: 300_000,
  });
  const defaultTag = settings.data?.default_creation_tag ?? null;

  const apply = useMutation({
    mutationFn: async () => {
      const missing = results.data?.missing ?? [];
      const movements = await Promise.all(
        selected
          .filter((sid) => missing.some((m) => m.id === sid))
          .map(async (sid) => {
            const c = customizations[sid] ?? {};
            const payload: ApplyMovementRequest = { movement_id: sid };
            if (c.counterpartyId != null) payload.counterparty_ff3_id = c.counterpartyId;
            if (c.description) payload.description = c.description;
            if (c.category) payload.category = c.category;
            if (c.budget) payload.budget = c.budget;
            if (c.tags) payload.tags = c.tags;
            if (c.notes) payload.notes = c.notes;
            if (c.attachment) {
              payload.attachment = {
                filename: c.attachment.filename,
                content_base64: await fileToBase64(c.attachment.file),
              };
            }
            return payload;
          }),
      );
      return postApply(id, { movements });
    },
    onSuccess: (res) => {
      toast.success(
        <ul className="meta">
          <li>{t('results.applyCreated', { count: res.created_count })}</li>
          <li>{t('results.applyAlready', { count: res.already_created_count })}</li>
          <li>{t('results.applyFailed', { count: res.failed_count })}</li>
        </ul>,
      );
      const failed = res.reports.filter((r) => r.outcome === 'created' && r.reason);
      if (failed.length > 0) {
        toast.warning(t('results.attachmentWarning', { reason: failed[0].reason }));
      }
      void qc.invalidateQueries({ queryKey: queryKeys.results(id) });
      void qc.invalidateQueries({ queryKey: queryKeys.import(id) });
      void qc.invalidateQueries({ queryKey: queryKeys.imports });
      setReports({ batchId: id, items: res.reports });
      setSelected([]);
      setCustomizations({});
      setConfirmOpen(false);
    },
    onError: (e) => toast.error(e.message),
  });

  // Optimistic overlay: movement id -> requested ignored state, until the refetched results agree.
  const [pendingIgnore, setPendingIgnore] = useState<Record<number, boolean>>({});
  const settleIgnore = (mid: number, ignored: boolean) =>
    setPendingIgnore((p) => {
      if (p[mid] !== ignored) return p;
      const next = { ...p };
      delete next[mid];
      return next;
    });
  type IgnoreVars = { mid: number; ignored: boolean; description: string };
  const ignore = useMutation({
    // One queue per batch: an Undo never reaches the backend before the request it reverts.
    scope: { id: `ignore-${id}` },
    mutationFn: ({ mid, ignored }: IgnoreVars) => postIgnore(id, mid, ignored),
    onSuccess: async (_res, { mid, ignored, description }) => {
      toast(t(ignored ? 'results.ignoredToast' : 'results.unignoredToast', { description }), {
        id: `ignore-${mid}`,
        duration: 8000,
        action: {
          label: t('results.undo'),
          onClick: () => toggleIgnore({ mid, ignored: !ignored, description }),
        },
      });
      // Settle only on a refetch that worked: a failed GET leaves a stale cache the overlay must keep hiding.
      await qc.invalidateQueries({ queryKey: queryKeys.results(id) }, { throwOnError: true }).then(
        () => settleIgnore(mid, ignored),
        () => {},
      );
    },
    onError: (e, { mid, ignored }) => {
      moveWithTransition(`mv-${mid}`, () => settleIgnore(mid, ignored));
      toast.error(e.message);
      void qc.invalidateQueries({ queryKey: queryKeys.results(id) });
    },
  });
  function toggleIgnore(vars: IgnoreVars) {
    // The clicked button unmounts with its row: refocus the same button in the row now at its index.
    const btn = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const table = btn?.closest('[role="table"]');
    const row = btn?.closest('[data-vt-key]');
    const index = table && row ? [...table.querySelectorAll('[data-vt-key]')].indexOf(row) : -1;
    const label = btn?.getAttribute('aria-label');
    moveWithTransition(
      `mv-${vars.mid}`,
      () => {
        setPendingIgnore((p) => ({ ...p, [vars.mid]: vars.ignored }));
        setSelected((s) => s.filter((x) => x !== vars.mid));
      },
      () => {
        const rows = table?.isConnected ? table.querySelectorAll('[data-vt-key]') : [];
        const next = index >= 0 ? rows[Math.min(index, rows.length - 1)] : undefined;
        const target =
          next?.querySelector<HTMLElement>(`button[aria-label="${label}"]`) ??
          document.getElementById('results-missing-table') ??
          document.querySelector<HTMLElement>('[role="tab"][data-state="active"]');
        target?.focus();
      },
    );
    ignore.mutate(vars);
  }

  const headerTitle = useMemo(
    () => (
      <span title={detail.data?.batch.filename}>
        {t('results.title')} - {detail.data?.batch.filename ?? `#${id}`}
      </span>
    ),
    [t, id, detail.data?.batch.filename],
  );
  const headerActions = useMemo(
    () => (
      <Button variant="outline" onClick={() => openRerun(id)}>
        {t('imports.rerun')}
      </Button>
    ),
    [openRerun, id, t],
  );
  usePageHeader({ title: headerTitle, actions: headerActions });
  const allAccounts = useMemo(
    () => [
      ...(accounts.data ?? []),
      ...(revenueAccounts.data ?? []),
      ...(expenseAccounts.data ?? []),
    ],
    [accounts.data, revenueAccounts.data, expenseAccounts.data],
  );
  const statementRange = useMemo(() => {
    const mv = detail.data?.movements ?? [];
    if (mv.length === 0) return null;
    const dates = mv.map((m) => m.date).sort();
    return { min: dates[0], max: dates[dates.length - 1] };
  }, [detail.data]);
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
  // A failed refetch keeps the cached data (and the optimistic overlay) on screen; the global toast reports it.
  if (!results.data || !detail.data) {
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
  const isOutsidePeriod = (d: string) =>
    !!statementRange && (d < statementRange.min || d > statementRange.max);
  const extraRows = results.data.extra;
  // Same order as the backend (line_no), so a moved row lands where the refetch will put it.
  const byLine = (a: { line_no: number }, b: { line_no: number }) => a.line_no - b.line_no;
  const missingRows = [
    ...results.data.missing.filter((m) => pendingIgnore[m.id] !== true),
    ...results.data.ignored.filter((m) => pendingIgnore[m.id] === false),
  ].sort(byLine);
  const ignoredRows = [
    ...results.data.ignored.filter((m) => pendingIgnore[m.id] !== false),
    ...results.data.missing.filter((m) => pendingIgnore[m.id] === true),
  ].sort(byLine);
  const counts = {
    matched: results.data.matched.length,
    missing: missingRows.length,
    extra: extraRows.filter((t) => !isOutsidePeriod(t.date)).length,
  };
  const activeSelected = selected.filter((sid) => missingRows.some((m) => m.id === sid));
  const currency = detail.data?.batch.currency ?? 'EUR';
  const tabs: Tab[] = ['matched', 'missing', 'extra'];
  const movementSummaries = selected
    .filter((sid) => missingRows.some((m) => m.id === sid))
    .map((sid) => {
      const m = missingRows.find((x) => x.id === sid)!;
      return {
        id: sid,
        description: m.description,
        amountCents: m.amount_cents,
        customization: customizations[sid] ?? {},
      };
    });
  const editingMovement =
    editingId != null ? (missingRows.find((m) => m.id === editingId) ?? null) : null;

  function handleBulkApply(fields: BulkFields) {
    setCustomizations((prev) => applyBulkFillOnly(prev, activeSelected, fields));
    setBulkOpen(false);
  }

  return (
    <div className="space-y-4">
      <Tabs value={tab} onValueChange={(v) => setTab(v as Tab)} className="gap-4">
        <TabsList
          variant="line"
          className="sticky top-0 z-[5] [view-transition-name:results-tabs] w-full justify-start rounded-none border-b border-border-subtle bg-canvas py-1"
        >
          {tabs.map((tb) => (
            <TabsTrigger key={tb} value={tb} className={TAB_ACCENT[tb]}>
              {t(`results.${tb}`)} ({counts[tb]})
            </TabsTrigger>
          ))}
        </TabsList>
        <TabsContent value="matched">
          {counts.matched === 0 ? (
            <EmptyState title={t('results.matched')} />
          ) : (
            <>
              <div role="table" aria-label={t('results.matched')} className={TABLE_WRAPPER}>
                <VirtualList
                  count={counts.matched}
                  header={<TableHeader group="matched" />}
                  renderItem={(i) => {
                    const { movement, ff3 } = results.data.matched[i];
                    const last = i === counts.matched - 1;
                    return (
                      <div
                        role="row"
                        className={`grid ${GRID_COLS.matched} items-center gap-3 px-4 py-3 text-sm transition-colors ${
                          last ? '' : 'border-b border-border-subtle'
                        } ${GROUP_ACCENT.matched.rowHover}`}
                      >
                        <span role="cell" className={CELL.matched.date}>
                          {formatDate(movement.date, i18n.language)}
                        </span>
                        <span
                          role="cell"
                          className={`min-w-0 truncate ${CELL.matched.desc}`}
                          title={movement.description}
                        >
                          {movement.description}
                        </span>
                        <span
                          role="cell"
                          className={`min-w-0 truncate ${CELL.matched.ff3}`}
                          title={ff3.description}
                        >
                          {ff3.description}
                        </span>
                        <div role="cell" className={`text-right ${CELL.matched.amount}`}>
                          <MoneyText cents={movement.amount_cents} currency={currency} />
                        </div>
                        <div role="cell" className={`flex justify-center ${CELL.matched.status}`}>
                          <StatusPill status={movement.status} />
                        </div>
                      </div>
                    );
                  }}
                />
              </div>
            </>
          )}
        </TabsContent>
        <TabsContent value="missing">
          {counts.missing === 0 ? (
            <EmptyState title={t('results.missing')} />
          ) : (
            <>
              <div
                id="results-missing-table"
                role="table"
                aria-label={t('results.missing')}
                tabIndex={-1}
                className={TABLE_WRAPPER}
              >
                <div className="flex flex-wrap items-center justify-between gap-2 border-b border-border-subtle px-4 py-2">
                  <label className="flex items-center gap-2 whitespace-nowrap text-sm text-text-2">
                    <span className="flex w-8 items-center justify-center">
                      <Checkbox
                        aria-label={t('results.selectAll')}
                        checked={activeSelected.length === counts.missing && counts.missing > 0}
                        onCheckedChange={() =>
                          setSelected(
                            activeSelected.length === counts.missing
                              ? []
                              : missingRows.map((m) => m.id),
                          )
                        }
                      />
                    </span>
                    {t('results.selectAll')}
                  </label>
                  <div className="flex flex-wrap gap-2">
                    <Button
                      variant="outline"
                      disabled={activeSelected.length === 0}
                      onClick={() => {
                        if (activeSelected.length === 1) {
                          setEditingId(activeSelected[0]);
                        } else {
                          setBulkOpen(true);
                        }
                      }}
                    >
                      {t('results.personalize')} ({activeSelected.length})
                    </Button>
                    <Button
                      disabled={activeSelected.length === 0}
                      onClick={() => setConfirmOpen(true)}
                    >
                      {t('results.apply')} ({activeSelected.length})
                    </Button>
                  </div>
                </div>
                <VirtualList
                  count={counts.missing}
                  header={<TableHeader group="missing" />}
                  renderItem={(i) => {
                    const m = missingRows[i];
                    return (
                      <MissingRow
                        movement={m}
                        currency={currency}
                        selected={activeSelected.includes(m.id)}
                        isLast={i === counts.missing - 1}
                        report={
                          reports.batchId === id
                            ? reports.items.find((r) => r.movement_id === m.id)
                            : undefined
                        }
                        onToggle={(mid) =>
                          setSelected((s) =>
                            s.includes(mid) ? s.filter((x) => x !== mid) : [...s, mid],
                          )
                        }
                        onEdit={() => setEditingId(m.id)}
                        onIgnore={(mid) =>
                          toggleIgnore({ mid, ignored: true, description: m.description })
                        }
                      />
                    );
                  }}
                />
              </div>
            </>
          )}
        </TabsContent>
        <TabsContent value="extra">
          {extraRows.length === 0 ? (
            <EmptyState title={t('results.extra')} />
          ) : (
            <>
              <div role="table" aria-label={t('results.extra')} className={TABLE_WRAPPER}>
                <VirtualList
                  count={extraRows.length}
                  header={<TableHeader group="extra" />}
                  renderItem={(i) => (
                    <ExtraRow
                      tx={extraRows[i]}
                      currency={currency}
                      isLast={i === extraRows.length - 1}
                      outsidePeriod={isOutsidePeriod(extraRows[i].date)}
                    />
                  )}
                />
              </div>
            </>
          )}
        </TabsContent>
      </Tabs>
      <section aria-label={t('results.ignored')}>
        <h2 className="px-2 pt-2 pb-1 text-sm font-semibold text-text-2">
          {t('results.ignored')} ({ignoredRows.length})
        </h2>
        {ignoredRows.length === 0 ? (
          <p className="px-2 py-3 text-sm text-text-3">{t('results.ignoredEmpty')}</p>
        ) : (
          <div role="table" aria-label={t('results.ignored')} className={TABLE_WRAPPER}>
            <TableHeader group="ignored" />
            {ignoredRows.map((m, i) => (
              <IgnoredRow
                key={m.id}
                movement={m}
                currency={currency}
                isLast={i === ignoredRows.length - 1}
                onUnignore={() =>
                  toggleIgnore({ mid: m.id, ignored: false, description: m.description })
                }
              />
            ))}
          </div>
        )}
      </section>
      {editingMovement && (
        <EditMovementDialog
          movement={editingMovement}
          initial={customizations[editingMovement.id] ?? {}}
          defaultTag={defaultTag}
          currency={currency}
          accounts={allAccounts}
          tags={tags.data ?? []}
          categories={categories.data ?? []}
          budgets={budgets.data ?? []}
          onSave={(c) => {
            setCustomizations((prev) => ({ ...prev, [editingMovement.id]: c }));
            setEditingId(null);
          }}
          onClose={() => setEditingId(null)}
        />
      )}
      {bulkOpen && tab === 'missing' && (
        <BulkPersonalizeDialog
          count={activeSelected.length}
          defaultTag={defaultTag}
          onApply={handleBulkApply}
          onClose={() => setBulkOpen(false)}
        />
      )}
      {confirmOpen && tab === 'missing' && (
        <ConfirmCreateDialog
          open={confirmOpen}
          batchId={id}
          filename={detail.data?.batch.filename ?? ''}
          movements={movementSummaries}
          accounts={allAccounts}
          pending={apply.isPending}
          onConfirm={() => apply.mutate()}
          onClose={() => setConfirmOpen(false)}
        />
      )}
    </div>
  );
}
