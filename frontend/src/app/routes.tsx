import { createBrowserRouter } from 'react-router-dom';
import { Shell } from './layout/Shell';
import { DashboardPage } from '@/features/dashboard/DashboardPage';

export const router = createBrowserRouter([
  {
    element: <Shell />,
    children: [
      { path: '/', element: <DashboardPage /> },
      { path: '/imports', element: <div data-testid="page-imports" /> },
      { path: '/results/:batchId', element: <div data-testid="page-results" /> },
    ],
  },
]);
