import { useMemo, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { getAccounts, getImport, getResults, getSettings, postApply } from '@/shared/api/endpoints';
import type { ApplyMovementRequest, ApplyReport } from '@/shared/api/dto';
import { formatDate } from '@/shared/lib/dates';
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
import { ExtraRow } from './ExtraRow';
import { BulkPersonalizeSheet } from './BulkPersonalizeSheet';
import { ConfirmCreateDialog } from './ConfirmCreateDialog';
import { TableHeader } from './tableLayout';

type Tab = 'matched' | 'missing' | 'extra';

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
        t('results.applyRecap', {
          created: res.created_count,
          already: res.already_created_count,
          failed: res.failed_count,
        }),
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

  const headerTitle = useMemo(
    () => (
      <span className="truncate">
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
  const movementSummaries = selected
    .filter((sid) => results.data.missing.some((m) => m.id === sid))
    .map((sid) => {
      const m = results.data.missing.find((x) => x.id === sid)!;
      return {
        id: sid,
        description: m.description,
        amountCents: m.amount_cents,
        customization: customizations[sid] ?? {},
      };
    });

  function handleBulkApply(fields: BulkFields) {
    setCustomizations((prev) => applyBulkFillOnly(prev, activeSelected, fields));
    setBulkOpen(false);
  }

  return (
    <div className="space-y-4">
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
            <>
              <TableHeader group="matched" />
              <VirtualList
                count={counts.matched}
                renderItem={(i) => {
                  const { movement, ff3 } = results.data.matched[i];
                  return (
                    <div className="grid grid-cols-[7rem_1fr_1fr_8rem_6rem] items-center gap-2 border-b border-border-subtle py-3 text-sm">
                      <span>{formatDate(movement.date, i18n.language)}</span>
                      <span className="min-w-0 truncate" title={movement.description}>
                        {movement.description}
                      </span>
                      <span className="min-w-0 truncate" title={ff3.description}>
                        {ff3.description}
                      </span>
                      <MoneyText cents={movement.amount_cents} currency={currency} />
                      <StatusPill status={movement.status} />
                    </div>
                  );
                }}
              />
            </>
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
                <div className="flex gap-2">
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
              <TableHeader group="missing" />
              <VirtualList
                renderItem={(i) => {
                  const m = results.data.missing[i];
                  return (
                    <>
                      <MissingRow
                        movement={m}
                        currency={currency}
                        selected={activeSelected.includes(m.id)}
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
                        editing={editingId === m.id}
                        onEdit={() => setEditingId(editingId === m.id ? null : m.id)}
                        accounts={accounts.data ?? []}
                        tags={tags.data ?? []}
                        categories={categories.data ?? []}
                        budgets={budgets.data ?? []}
                        defaultTag={defaultTag}
                        onSave={(c) => {
                          setCustomizations((prev) => ({ ...prev, [m.id]: c }));
                          setEditingId(null);
                        }}
                        onCancel={() => setEditingId(null)}
                      />
                    </>
                  );
                }}
              />
            </>
          )}
        </TabsContent>
        <TabsContent value="extra">
          {counts.extra === 0 ? (
            <EmptyState title={t('results.extra')} />
          ) : (
            <>
              <TableHeader group="extra" />
              <VirtualList
                count={counts.extra}
                renderItem={(i) => <ExtraRow tx={results.data.extra[i]} currency={currency} />}
              />
            </>
          )}
        </TabsContent>
      </Tabs>
      {bulkOpen && tab === 'missing' && (
        <BulkPersonalizeSheet
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
          accounts={accounts.data ?? []}
          onConfirm={() => apply.mutate()}
          onClose={() => setConfirmOpen(false)}
        />
      )}
    </div>
  );
}
