-- Tablas independientes (sin FKs): cache de cuentas FF3, mappings CSV, audit de borrados.
-- Los JSON se almacenan como JSON nativo de MySQL (validación DB-level).

CREATE TABLE ff3_accounts (
    id          BIGINT       NOT NULL AUTO_INCREMENT PRIMARY KEY,
    ff3_id      INT          NOT NULL,
    name        VARCHAR(255) NOT NULL,
    type        VARCHAR(50)  NOT NULL,
    currency    CHAR(3)      NOT NULL,
    synced_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uq_ff3_accounts_ff3_id (ff3_id)
);

CREATE TABLE csv_mappings (
    id               BIGINT       NOT NULL AUTO_INCREMENT PRIMARY KEY,
    header_signature VARCHAR(255) NOT NULL,
    mapping_json     JSON         NOT NULL,
    created_at       DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uq_csv_mappings_header_signature (header_signature)
);

CREATE TABLE deleted_audit (
    id            BIGINT   NOT NULL AUTO_INCREMENT PRIMARY KEY,
    ff3_id        BIGINT   NOT NULL,
    snapshot_json JSON     NOT NULL,
    deleted_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
