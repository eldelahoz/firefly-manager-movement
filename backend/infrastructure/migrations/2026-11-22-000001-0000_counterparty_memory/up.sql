-- RF-08 v0.11: counterparty learned per batch account, direction and normalized description.
-- The counterparty is an existing account (id, name), a name (id when Firefly III returned it) or Cash.
CREATE TABLE counterparty_memory (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    account_ff3_id      BIGINT  NOT NULL,
    direction           TEXT    NOT NULL CHECK (direction IN ('in', 'out')),
    desc_key            TEXT    NOT NULL,
    counterparty_ff3_id BIGINT  NULL,
    counterparty_name   TEXT    NULL,
    is_cash             BOOLEAN NOT NULL DEFAULT FALSE,
    uses                BIGINT  NOT NULL DEFAULT 1,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (account_ff3_id, direction, desc_key)
);
