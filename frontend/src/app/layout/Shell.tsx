import { NavLink, Outlet } from 'react-router-dom';
import { FolderInput, LayoutDashboard, Settings } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ThemeSwitcher } from './ThemeSwitcher';
import { LocaleSwitcher } from './LocaleSwitcher';
import { HealthChip } from './HealthChip';

export function Shell() {
  const { t } = useTranslation();
  const items = [
    { to: '/', label: t('nav.dashboard'), icon: LayoutDashboard },
    { to: '/imports', label: t('nav.imports'), icon: FolderInput },
  ];
  return (
    <div className="min-h-screen bg-canvas text-text-1">
      <aside className="fixed inset-y-0 left-0 hidden w-[220px] flex-col gap-1 border-r border-border-subtle bg-surface p-4 md:flex">
        <span className="mb-4 px-2 text-lg font-bold">FFM</span>
        {items.map(({ to, label, icon: Icon }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `flex items-center gap-3 rounded-lg px-3 py-2.5 ${isActive ? 'bg-accent-soft text-accent' : 'text-text-2 hover:bg-canvas'}`
            }
          >
            <Icon className="size-5" />
            {label}
          </NavLink>
        ))}
        <span
          className="flex items-center gap-3 rounded-lg px-3 py-2.5 text-text-3"
          aria-disabled="true"
        >
          <Settings className="size-5" />
          {t('nav.settings')}
        </span>
      </aside>
      <div className="md:pl-[220px]">
        <header className="sticky top-0 z-10 flex h-16 items-center justify-between border-b border-border-subtle bg-surface px-4">
          <HealthChip />
          <div className="flex items-center gap-2">
            <LocaleSwitcher />
            <ThemeSwitcher />
          </div>
        </header>
        <main className="mx-auto max-w-6xl p-4 pb-24 md:pb-8">
          <Outlet />
        </main>
      </div>
      <nav className="fixed inset-x-0 bottom-0 z-10 flex h-16 border-t border-border-subtle bg-surface md:hidden">
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
        <span className="flex flex-1 flex-col items-center justify-center gap-1 text-text-3">
          <Settings className="size-5" />
          <span className="text-xs">{t('nav.settings')}</span>
        </span>
      </nav>
    </div>
  );
}
