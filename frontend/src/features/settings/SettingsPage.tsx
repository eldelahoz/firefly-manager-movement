import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useMutation, useQuery } from '@tanstack/react-query';
import { toast } from 'sonner';
import { useTheme } from '@/app/ThemeProvider';
import { getSettings, putSettings, postVerifySettings, getTags } from '@/shared/api/endpoints';
import { queryKeys } from '@/shared/lib/keys';
import { queryClient } from '@/shared/lib/queryClient';
import { Button } from '@/shared/ui/button';
import { Input } from '@/shared/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select';
import { Switch } from '@/shared/ui/switch';

const LANGUAGES = [
  { value: 'en', label: 'English' },
  { value: 'es', label: 'Español' },
  { value: 'fr', label: 'Français' },
] as const;

export function SettingsPage() {
  const { t, i18n } = useTranslation();
  const { theme, setTheme } = useTheme();
  const current = i18n.resolvedLanguage ?? i18n.language;

  // --- Connection state ---
  const settings = useQuery({ queryKey: queryKeys.settings, queryFn: getSettings });
  const tags = useQuery({ queryKey: queryKeys.tags, queryFn: getTags, staleTime: 300_000 });

  const [urlDraft, setUrlDraft] = useState<string | null>(null);
  const [tokenDraft, setTokenDraft] = useState('');
  const [tagDraft, setTagDraft] = useState<string | null>(null);

  const url = urlDraft ?? settings.data?.ff3_url ?? '';
  const tag = tagDraft ?? settings.data?.default_creation_tag ?? 'none';

  const save = useMutation({
    mutationFn: () =>
      putSettings({
        ff3_url: url.trim(),
        ff3_token: tokenDraft.trim() || undefined,
        default_creation_tag: tag === 'none' ? null : tag,
      }),
    onSuccess: (s) => {
      toast.success(t('settings.saved'));
      setTagDraft(s.default_creation_tag ?? 'none');
    },
    onError: (e) => toast.error(e.message),
  });

  const verify = useMutation({
    mutationFn: () => postVerifySettings({ ff3_url: url.trim(), ff3_token: tokenDraft.trim() }),
    onSuccess: (r) => toast.success(t('settings.connectionOk', { version: r.version })),
    onError: (e) => toast.error(e.message),
  });

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">{t('settings.title')}</h1>

      {/* ── Connection ────────────────────────────── */}
      <section className="rounded-xl border border-border-subtle bg-surface p-4 space-y-3">
        <h2 className="font-semibold">{t('settings.connection')}</h2>

        <div className="space-y-1">
          <label htmlFor="ff3-url" className="text-sm font-medium">
            {t('settings.ff3Url')}
          </label>
          <Input
            id="ff3-url"
            value={url}
            onChange={(e) => setUrlDraft(e.target.value)}
            placeholder="http://localhost:8080"
          />
        </div>

        <div className="space-y-1">
          <label htmlFor="ff3-token" className="text-sm font-medium">
            {t('settings.ff3Token')}
          </label>
          <Input
            id="ff3-token"
            type="password"
            value={tokenDraft}
            onChange={(e) => setTokenDraft(e.target.value)}
            placeholder="••••••••"
          />
          {settings.data?.has_token && tokenDraft === '' && (
            <p className="text-xs text-muted-foreground">{t('settings.ff3TokenKeep')}</p>
          )}
        </div>

        <div className="space-y-1">
          <span className="text-sm font-medium">{t('settings.defaultTag')}</span>
          <Select value={tag} onValueChange={(v) => setTagDraft(v)}>
            <SelectTrigger className="w-[220px]" aria-label={t('settings.defaultTag')}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent position="popper">
              <SelectItem value="none">{t('settings.defaultTagNone')}</SelectItem>
              {tags.data?.map((name) => (
                <SelectItem key={name} value={name}>
                  {name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1">
          <span className="text-sm font-medium">{t('settings.enableOriginTrace')}</span>
          <Switch
            checked={settings.data?.enable_origin_trace ?? true}
            onCheckedChange={(checked) => {
              putSettings({
                ff3_url: settings.data?.ff3_url ?? '',
                default_creation_tag: settings.data?.default_creation_tag,
                enable_origin_trace: checked,
              }).then((s) => {
                queryClient.setQueryData(queryKeys.settings, s);
              });
            }}
          />
        </div>

        <div className="flex gap-2 pt-1">
          <Button variant="outline" disabled={verify.isPending} onClick={() => verify.mutate()}>
            {verify.isPending ? '…' : t('settings.testConnection')}
          </Button>
          <Button disabled={save.isPending} onClick={() => save.mutate()}>
            {save.isPending ? '…' : t('settings.save')}
          </Button>
        </div>
      </section>

      {/* ── Appearance ────────────────────────────── */}
      <section className="rounded-xl border border-border-subtle bg-surface p-4">
        <h2 className="font-semibold">{t('settings.appearance')}</h2>
        <div className="mt-3 flex gap-2">
          <Button
            variant={theme === 'light' ? 'default' : 'outline'}
            onClick={() => setTheme('light')}
          >
            {t('settings.themeLight')}
          </Button>
          <Button
            variant={theme === 'dark' ? 'default' : 'outline'}
            onClick={() => setTheme('dark')}
          >
            {t('settings.themeDark')}
          </Button>
        </div>
      </section>

      {/* ── Language ──────────────────────────────── */}
      <section className="rounded-xl border border-border-subtle bg-surface p-4">
        <h2 className="font-semibold">{t('settings.language')}</h2>
        <Select
          value={current}
          onValueChange={(v) => {
            void i18n.changeLanguage(v);
          }}
        >
          <SelectTrigger className="mt-3 w-[180px]" aria-label={t('settings.language')}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent position="popper">
            {LANGUAGES.map((l) => (
              <SelectItem key={l.value} value={l.value}>
                {l.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </section>
    </div>
  );
}
