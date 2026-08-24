import { api, uploadCsv } from './client';
import type {
  ApplyRequest,
  ApplyResponse,
  BatchSummary,
  Ff3Account,
  Ff3TxDTO,
  HealthResponse,
  ImportDetail,
  ReconcileRequest,
  ReconcileResponse,
  ResultsResponse,
  UploadResponse,
} from './dto';

export const getHealth = () => api.get<HealthResponse>('/health');
export const getImports = () => api.get<BatchSummary[]>('/api/imports');
export const getImport = (id: number) => api.get<ImportDetail>(`/api/imports/${id}`);
export const putMapping = (
  id: number,
  body: { headers: string[]; date_column: string; amount_column: string; description_column: string },
) => api.put<{ mapping_id: number }>(`/api/imports/${id}/mapping`, body);
export const postReconcile = (id: number, body: ReconcileRequest) =>
  api.post<ReconcileResponse>(`/api/imports/${id}/reconcile`, body);
export const getResults = (id: number) => api.get<ResultsResponse>(`/api/imports/${id}/results`);
export const postIgnore = (batchId: number, movementId: number) =>
  api.post<{ movement_id: number; status: 'ignored' }>(
    `/api/imports/${batchId}/movements/${movementId}/ignore`,
  );
export const postApply = (batchId: number, body: ApplyRequest) =>
  api.post<ApplyResponse>(`/api/imports/${batchId}/apply`, body);
export const getAccounts = () => api.get<Ff3Account[]>('/api/firefly/accounts');
export const postAccountTransactions = (id: string, start: string, end: string) =>
  api.post<Ff3TxDTO[]>(`/api/firefly/accounts/${id}/transactions`, { start, end });
export const deleteTransaction = (id: string) =>
  api.del<{ ff3_id: string; status: 'deleted' }>(`/api/firefly/transactions/${id}`);
export { uploadCsv };
export type { UploadResponse };
