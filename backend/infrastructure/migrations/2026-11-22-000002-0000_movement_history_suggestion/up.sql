-- RF-08 v0.11: Firefly III history suggestion of each movement, set at reconcile time.
-- Nullable: movements reconciled before this migration have no suggestion.
ALTER TABLE movements ADD COLUMN history_ff3_id BIGINT;
ALTER TABLE movements ADD COLUMN history_name TEXT;
ALTER TABLE movements ADD COLUMN history_cash BOOLEAN;
