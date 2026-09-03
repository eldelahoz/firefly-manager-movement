export interface HealthResponse {
  status: 'ok' | 'degraded';
  ff3?: { reachable: boolean; version?: string };
}

export interface MovementDTO {
  id: number;
  line_no: number;
  date: string;
  amount_cents: number;
  description: string;
  status: 'imported' | 'matched' | 'missing' | 'created' | 'ignored';
}

export interface UploadResponse {
  batch_id: number;
  filename: string;
  file_hash: string;
  headers: string[];
  detected_delimiter: string;
  total_rows: number;
  valid_count: number;
  invalid_count: number;
  mapping_used: {
    date_column: string;
    amount_column: string;
    description_column: string;
    origin_account_column?: string | null;
  } | null;
  origin_account_values: string[];
  valid: { line_no: number; date: string; amount_cents: number; description: string }[];
  invalid: { line_no: number; reason: string; raw: string }[];
}

export interface BatchSummary {
  id: number;
  filename: string;
  file_hash: string;
  format: string;
  currency: string;
  status: string;
  created_at: string;
  account_id: number | null;
  account_name: string | null;
  movements_count: number;
}

export interface ImportDetail {
  batch: BatchSummary;
  movements: MovementDTO[];
}

export interface Ff3Account {
  id: number;
  name: string;
  type: string;
  currency: string;
  iban?: string | null;
  account_number?: string | null;
}

export interface Ff3TxDTO {
  id: number;
  date: string;
  amount_cents: number;
  description: string;
  status?: 'matched' | 'duplicate' | 'extra' | null;
}

export interface Ff3CachedTxDTO {
  id: number;
  ff3_id: number;
  date: string;
  amount_cents: number;
  description: string;
  is_duplicate: boolean;
  created_at: string;
}

export interface ReconcileRequest {
  ff3_account_id: number;
  window_start: string;
  window_end: string;
  tolerance_days?: number;
}

export interface ReconcileResponse {
  reconciliation_id: number;
  batch_id: number;
  tolerance_days: number;
  window_start: string;
  window_end: string;
  matched_count: number;
  missing_count: number;
  extra_count: number;
  duplicate_count: number;
}

export interface ResultsResponse {
  reconciliation_id: number;
  matched: { movement: MovementDTO; ff3: Ff3CachedTxDTO }[];
  missing: MovementDTO[];
  extra: Ff3CachedTxDTO[];
}

export interface AttachmentPayload {
  filename: string;
  content_base64: string;
}

export interface ApplyMovementRequest {
  movement_id: number;
  counterparty_ff3_id: number;
  description?: string;
  category?: string;
  budget?: string;
  tags?: string[];
  notes?: string;
  attachment?: AttachmentPayload;
}

export type ApplyRequestV2 = { movements: ApplyMovementRequest[] };

export interface ApplyReport {
  movement_id: number;
  outcome: 'created' | 'already_created' | 'failed';
  ff3_id?: number;
  reason?: string;
}

export interface ApplyResponse {
  reports: ApplyReport[];
  created_count: number;
  already_created_count: number;
  failed_count: number;
  reconciliation_id?: number;
}

export interface SettingsResponse {
  ff3_url: string;
  has_token: boolean;
  default_creation_tag: string | null;
}

export interface SettingsPutRequest {
  ff3_url: string;
  ff3_token?: string;
  default_creation_tag?: string | null;
}

export interface VerifyResponse {
  version: string;
}
