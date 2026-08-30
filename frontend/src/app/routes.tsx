import { createBrowserRouter } from 'react-router-dom';
import { Shell } from './layout/Shell';
import { DashboardPage } from '@/features/dashboard/DashboardPage';
import { ImportsPage } from '@/features/imports/ImportsPage';
import { ResultsPage } from '@/features/results/ResultsPage';
import { SettingsPage } from '@/features/settings/SettingsPage';
import { ReconcileProvider } from './reconcileContext';

export const router = createBrowserRouter([
  {
    element: (
      <ReconcileProvider>
        <Shell />
      </ReconcileProvider>
    ),
    children: [
      { path: '/', element: <DashboardPage /> },
      { path: '/imports', element: <ImportsPage /> },
      { path: '/results/:batchId', element: <ResultsPage /> },
      { path: '/settings', element: <SettingsPage /> },
    ],
  },
]);
