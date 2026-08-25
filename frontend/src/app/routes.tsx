import { createBrowserRouter } from 'react-router-dom';
import { Shell } from './layout/Shell';
import { DesignPage } from './DesignPage';

export const router = createBrowserRouter([
  {
    element: <Shell />,
    children: [
      { path: '/', element: <div data-testid="page-dashboard" /> },
      { path: '/imports', element: <div data-testid="page-imports" /> },
      { path: '/results/:batchId', element: <div data-testid="page-results" /> },
      { path: '/design', element: <DesignPage /> },
    ],
  },
]);
