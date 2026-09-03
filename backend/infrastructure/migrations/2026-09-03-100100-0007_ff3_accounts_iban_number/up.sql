-- Origin-account matching (RF-04 v0.7): match bank exports by IBAN,
-- account number or name against the cached accounts.

ALTER TABLE ff3_accounts
    ADD COLUMN iban VARCHAR(64) NULL AFTER currency,
    ADD COLUMN account_number VARCHAR(255) NULL AFTER iban;
