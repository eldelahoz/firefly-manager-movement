-- Split transactions matched per split (RF-06): one cache row per matched journal.
-- 0 = the whole transaction group (NOT NULL keeps the unique key strict).

ALTER TABLE ff3_transactions
    ADD COLUMN ff3_journal_id BIGINT NOT NULL DEFAULT 0 AFTER ff3_id,
    DROP INDEX uq_ff3_transactions_run_ff3,
    ADD UNIQUE KEY uq_ff3_transactions_run_ff3_journal (reconciliation_id, ff3_id, ff3_journal_id)
