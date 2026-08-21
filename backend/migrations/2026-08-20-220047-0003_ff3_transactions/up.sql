-- ff3_transactions: per-run cache of the examined FF3 txs (extra group + duplicate flag).

CREATE TABLE ff3_transactions (
    id                BIGINT      NOT NULL AUTO_INCREMENT PRIMARY KEY,
    reconciliation_id BIGINT      NOT NULL,
    ff3_id             BIGINT      NOT NULL,
    date               DATE       NOT NULL,
    amount_cents       BIGINT      NOT NULL,
    description        TEXT       NOT NULL,
    is_duplicate       BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at         DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_ff3_transactions_reconciliation FOREIGN KEY (reconciliation_id) REFERENCES reconciliations (id),
    UNIQUE KEY uq_ff3_transactions_run_ff3 (reconciliation_id, ff3_id)
);
