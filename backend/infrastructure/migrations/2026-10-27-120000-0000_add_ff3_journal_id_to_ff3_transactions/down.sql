-- Fails while a run caches several splits of one group (no silent data loss).

ALTER TABLE ff3_transactions
    DROP INDEX uq_ff3_transactions_run_ff3_journal,
    ADD UNIQUE KEY uq_ff3_transactions_run_ff3 (reconciliation_id, ff3_id),
    DROP COLUMN ff3_journal_id
