-- RF-07 v0.11 / RF-14 v0.2: opposite-side account of each snapshot row.
-- Nullable: snapshots saved before this migration stay NULL.
ALTER TABLE ff3_transactions ADD COLUMN counterparty_name TEXT;
ALTER TABLE ff3_transactions ADD COLUMN counterparty_type TEXT;
