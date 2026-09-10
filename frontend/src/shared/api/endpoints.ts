import { api, uploadCsv } from './client';
import type {
  ApplyRequestV2,
  ApplyResponse,
  BatchSummary,
  Ff3Account,
  Ff3TxDTO,
  HealthResponse,
  ImportDetail,
  ReconcileRequest,
  ReconcileResponse,
  ResultsResponse,
  SettingsPutRequest,
  SettingsResponse,
  UploadResponse,
  VerifyResponse,
} from './dto';

export const getHealth = () => api.get<HealthResponse>('/health');
export const getImports = () => api.get<BatchSummary[]>('/api/imports');
export const getImport = (id: number) => api.get<ImportDetail>(`/api/imports/${id}`);
export const putMapping = (
  id: number,
  body: {
    headers: string[];
    date_column: string;
    amount_column: string;
    description_column: string;
  },
) => api.put<{ mapping_id: number }>(`/api/imports/${id}/mapping`, body);
export const postReconcile = (id: number, body: ReconcileRequest) =>
  api.post<ReconcileResponse>(`/api/imports/${id}/reconcile`, body);
export const getResults = (id: number) => api.get<ResultsResponse>(`/api/imports/${id}/results`);
export const postIgnore = (batchId: number, movementId: number, ignored: boolean) =>
  api.post<{ movement_id: number; status: string }>(
    `/api/imports/${batchId}/movements/${movementId}/ignore`,
    { ignored },
  );
export const postApply = (batchId: number, body: ApplyRequestV2) =>
  api.post<ApplyResponse>(`/api/imports/${batchId}/apply`, body);
export const deleteImport = (id: number) =>
  api.del<{ id: number; status: 'deleted' }>(`/api/imports/${id}`);
export const getAccounts = () => api.get<Ff3Account[]>('/api/firefly/accounts');
export const getAccountsByType = (type: 'asset' | 'expense' | 'revenue') =>
  api.get<Ff3Account[]>(`/api/firefly/accounts?type=${type}`);
export const postAccountTransactions = (id: string, start?: string, end?: string) =>
  api.post<Ff3TxDTO[]>(`/api/firefly/accounts/${id}/transactions`, {
    ...(start ? { start } : {}),
    ...(end ? { end } : {}),
  });
export const deleteTransaction = (id: number) =>
  api.del<{ ff3_id: number; status: 'deleted' }>(`/api/firefly/transactions/${id}`);
export const getSettings = () => api.get<SettingsResponse>('/api/settings');
export const putSettings = (body: SettingsPutRequest) =>
  api.put<SettingsResponse>('/api/settings', body);
export const postVerifySettings = (body: { ff3_url: string; ff3_token: string }) =>
  api.post<VerifyResponse>('/api/settings/verify', body);
export const getTags = () => api.get<string[]>('/api/firefly/tags');
export const getCategories = () => api.get<string[]>('/api/firefly/categories');
export const getBudgets = () => api.get<string[]>('/api/firefly/budgets');
export { uploadCsv };
export type { UploadResponse };
