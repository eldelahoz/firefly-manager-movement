-- batches: batch of imported movements (immutable, RF-09).
-- account_id FK → ff3_accounts (nullable until the target account is chosen).

CREATE TABLE batches (
    id         BIGINT       NOT NULL AUTO_INCREMENT PRIMARY KEY,
    filename   VARCHAR(255) NOT NULL,
    file_hash  VARCHAR(255) NOT NULL,
    format     VARCHAR(50)  NOT NULL,
    currency   CHAR(3)      NOT NULL,
    account_id BIGINT       NULL,
    status     VARCHAR(20)  NOT NULL,
    created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_batches_account FOREIGN KEY (account_id) REFERENCES ff3_accounts (id)
);
