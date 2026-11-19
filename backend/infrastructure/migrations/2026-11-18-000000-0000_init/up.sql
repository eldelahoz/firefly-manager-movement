-- Initial SQLite schema (replaces the 13 MySQL migrations; no data carried over).
-- INTEGER PRIMARY KEY AUTOINCREMENT = 64-bit rowid, never reused after deletes (RF-17). JSON columns are TEXT (diesel Json on SQLite).
-- foreign_keys is enabled per connection by the pool.

CREATE TABLE ff3_accounts (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    ff3_id         INTEGER NOT NULL UNIQUE,
    name           TEXT    NOT NULL,
    type           TEXT    NOT NULL,
    currency       TEXT    NOT NULL,
    iban           TEXT    NULL,
    account_number TEXT    NULL,
    synced_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE csv_mappings (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    header_signature TEXT    NOT NULL UNIQUE,
    mapping_json     TEXT    NOT NULL,
    created_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE deleted_audit (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    ff3_id        BIGINT  NOT NULL,
    snapshot_json TEXT    NOT NULL,
    deleted_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- RF-16: duplicate filename check is case-insensitive (NOCASE folds ASCII only).
CREATE TABLE batches (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    filename           TEXT    NOT NULL COLLATE NOCASE,
    file_hash          TEXT    NOT NULL,
    format             TEXT    NOT NULL,
    currency           TEXT    NOT NULL,
    account_id         BIGINT  NULL REFERENCES ff3_accounts (id),
    status             TEXT    NOT NULL,
    created_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    headers_json       TEXT    NOT NULL DEFAULT '[]',
    detected_delimiter TEXT    NOT NULL DEFAULT ','
);

CREATE TABLE movements (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_id     BIGINT  NOT NULL REFERENCES batches (id),
    line_no      INTEGER NOT NULL,
    date         DATE    NOT NULL,
    amount_cents BIGINT  NOT NULL CHECK (amount_cents <> 0),
    description  TEXT    NOT NULL,
    raw_json     TEXT    NOT NULL,
    status       TEXT    NOT NULL,
    created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (batch_id, line_no)
);

CREATE TABLE reconciliations (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_id       BIGINT  NOT NULL REFERENCES batches (id),
    tolerance_days INTEGER NOT NULL,
    window_start   DATE    NOT NULL,
    window_end     DATE    NOT NULL,
    matched_count  INTEGER NOT NULL,
    missing_count  INTEGER NOT NULL,
    extra_count    INTEGER NOT NULL,
    ran_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ff3_journal_id 0 = the whole transaction group (NOT NULL keeps the unique key strict, RF-06).
CREATE TABLE ff3_transactions (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    reconciliation_id BIGINT  NOT NULL REFERENCES reconciliations (id),
    ff3_id            BIGINT  NOT NULL,
    ff3_journal_id    BIGINT  NOT NULL DEFAULT 0,
    date              DATE    NOT NULL,
    amount_cents      BIGINT  NOT NULL,
    description       TEXT    NOT NULL,
    is_duplicate      BOOLEAN NOT NULL DEFAULT FALSE,
    created_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (reconciliation_id, ff3_id, ff3_journal_id)
);

-- 1-to-1 per run is guaranteed by the two unique keys.
CREATE TABLE matches (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    reconciliation_id  BIGINT  NOT NULL REFERENCES reconciliations (id),
    movement_id        BIGINT  NOT NULL REFERENCES movements (id),
    ff3_transaction_id BIGINT  NOT NULL REFERENCES ff3_transactions (id),
    matched_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (reconciliation_id, movement_id),
    UNIQUE (reconciliation_id, ff3_transaction_id)
);

-- UNIQUE(movement_id) = creation idempotency (NFR-05).
CREATE TABLE creations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    movement_id BIGINT  NOT NULL UNIQUE REFERENCES movements (id),
    ff3_id      BIGINT  NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- RF-19: single row (id = 1). ff3_token is write-only (NFR-02). updated_at is set by the upsert.
CREATE TABLE app_settings (
    id                   INTEGER PRIMARY KEY,
    ff3_url              TEXT    NOT NULL,
    ff3_token            TEXT    NOT NULL,
    default_creation_tag TEXT    NULL,
    updated_at           TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    enable_origin_trace  BOOLEAN NOT NULL DEFAULT TRUE,
    default_date_format  TEXT    NULL
);

CREATE INDEX idx_movements_batch_status ON movements (batch_id, status);
CREATE INDEX idx_movements_batch_date ON movements (batch_id, date);
CREATE INDEX idx_ff3_transactions_run_amount ON ff3_transactions (reconciliation_id, amount_cents);
CREATE INDEX idx_matches_movement ON matches (movement_id);
CREATE INDEX idx_reconciliations_batch ON reconciliations (batch_id);
