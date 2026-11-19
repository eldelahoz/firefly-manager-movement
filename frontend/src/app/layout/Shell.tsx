import { useState } from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import {
  FolderInput,
  LayoutDashboard,
  PanelLeftClose,
  PanelLeftOpen,
  Settings,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ThemeSwitcher } from './ThemeSwitcher';
import { HealthChip } from './HealthChip';
import { ImportWizardProvider } from '@/app/importWizardContext';
import { PageHeaderProvider } from '@/shared/ui/PageHeader';

const SIDEBAR_KEY = 'ffm.sidebar';
function readCollapsed() {
  try {
    return localStorage.getItem(SIDEBAR_KEY) === 'collapsed';
  } catch {
    return false; // storage blocked (private mode): default to expanded
  }
}

export function Shell() {
  const { t } = useTranslation();
  const [collapsed, setCollapsed] = useState(readCollapsed);
  // Collapsed rail: keyboard focus gets a visible label too (title tooltips only show on hover).
  const [focusTip, setFocusTip] = useState<{ label: string; top: number } | null>(null);
  const toggle = () => {
    const next = !collapsed;
    setCollapsed(next);
    try {
      localStorage.setItem(SIDEBAR_KEY, next ? 'collapsed' : 'expanded');
    } catch {
      // per-viewer convenience only; ignore storage failures
    }
  };
  const items = [
    { to: '/', label: t('nav.dashboard'), icon: LayoutDashboard },
    { to: '/imports', label: t('nav.imports'), icon: FolderInput },
    { to: '/settings', label: t('nav.settings'), icon: Settings },
  ];
  return (
    <ImportWizardProvider>
      <div
        className="ffm-shell min-h-screen bg-canvas text-text-1"
        data-sidebar={collapsed ? 'collapsed' : 'expanded'}
      >
        <aside
          id="app-sidebar"
          className="ffm-sidebar fixed inset-y-0 left-0 hidden w-(--sidebar-w) flex-col overflow-hidden border-r border-border-subtle bg-surface p-3 md:flex"
        >
          <div className="mb-4 flex items-center">
            <span className="ffm-sidebar-label ffm-brand min-w-0 flex-1 truncate px-2.5 text-lg font-bold">
              FFM
            </span>
            <button
              type="button"
              onClick={toggle}
              aria-expanded={!collapsed}
              aria-controls="app-sidebar"
              aria-label={t('nav.toggleSidebar')}
              title={t('nav.toggleSidebar')}
              className="flex size-10 shrink-0 items-center justify-center rounded-lg text-text-2 hover:bg-canvas"
            >
              {collapsed ? (
                <PanelLeftOpen className="size-5" />
              ) : (
                <PanelLeftClose className="size-5" />
              )}
            </button>
          </div>
          <nav className="flex flex-col gap-1">
            {items.map(({ to, label, icon: Icon }) => (
              <NavLink
                key={to}
                to={to}
                title={collapsed ? label : undefined}
                onFocus={(e) => {
                  if (!collapsed || !e.currentTarget.matches(':focus-visible')) return;
                  const r = e.currentTarget.getBoundingClientRect();
                  setFocusTip({ label, top: r.top + r.height / 2 });
                }}
                onBlur={() => setFocusTip(null)}
                className={({ isActive }) =>
                  `flex items-center gap-3 whitespace-nowrap rounded-lg px-2.5 py-2.5 ${isActive ? 'bg-accent-soft text-accent' : 'text-text-2 hover:bg-canvas'}`
                }
              >
                <Icon className="size-5 shrink-0" />
                <span className="ffm-sidebar-label">{label}</span>
              </NavLink>
            ))}
          </nav>
          <div className="mt-auto flex flex-col gap-2 border-t border-border-subtle pt-3">
            <HealthChip />
            <ThemeSwitcher />
          </div>
        </aside>
        {collapsed && focusTip ? (
          <span
            aria-hidden="true"
            className="pointer-events-none fixed left-[72px] z-50 hidden -translate-y-1/2 rounded-md bg-text-1 px-2 py-1 text-sm text-surface shadow-md md:block"
            style={{ top: focusTip.top }}
          >
            {focusTip.label}
          </span>
        ) : null}
        <div className="ffm-content md:pl-(--sidebar-w)">
          <PageHeaderProvider>
            <main className="mx-auto max-w-6xl p-4 pb-24 md:pb-8">
              <Outlet />
            </main>
          </PageHeaderProvider>
        </div>
        <nav className="fixed inset-x-0 bottom-0 z-10 [view-transition-name:mobile-nav] flex h-16 border-t border-border-subtle bg-surface md:hidden">
          {items.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                `flex flex-1 flex-col items-center justify-center gap-1 ${isActive ? 'text-accent' : 'text-text-2'}`
              }
            >
              <Icon className="size-5" />
              <span className="text-xs">{label}</span>
            </NavLink>
          ))}
        </nav>
      </div>
    </ImportWizardProvider>
  );
}
