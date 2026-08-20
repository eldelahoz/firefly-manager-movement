-- Índices secundarios (TRD §2): filtrado por estado/fecha de movimientos, búsqueda por monto en ff3_transactions.

CREATE INDEX idx_movements_batch_status ON movements (batch_id, status);
CREATE INDEX idx_movements_batch_date ON movements (batch_id, date);
CREATE INDEX idx_ff3_transactions_run_amount ON ff3_transactions (reconciliation_id, amount_cents);
