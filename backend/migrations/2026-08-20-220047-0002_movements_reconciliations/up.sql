-- movements: movimiento importado. amount_cents con signo (sens derivado), sin direction ni ff3_transaction_id.
-- reconciliations: snapshot congelado de un run (contadores de audit).

CREATE TABLE movements (
    id           BIGINT      NOT NULL AUTO_INCREMENT PRIMARY KEY,
    batch_id     BIGINT      NOT NULL,
    line_no      INT         NOT NULL,
    date         DATE        NOT NULL,
    amount_cents BIGINT      NOT NULL,
    description TEXT        NOT NULL,
    raw_json     JSON        NOT NULL,
    status       VARCHAR(20) NOT NULL,
    created_at   DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_movements_batch FOREIGN KEY (batch_id) REFERENCES batches (id),
    UNIQUE KEY uq_movements_batch_lineno (batch_id, line_no),
    CONSTRAINT ck_movements_amount_nonzero CHECK (amount_cents <> 0)
);

CREATE TABLE reconciliations (
    id             BIGINT   NOT NULL AUTO_INCREMENT PRIMARY KEY,
    batch_id       BIGINT   NOT NULL,
    tolerance_days INT      NOT NULL,
    window_start   DATE     NOT NULL,
    window_end     DATE     NOT NULL,
    matched_count  INT      NOT NULL,
    missing_count  INT      NOT NULL,
    extra_count    INT      NOT NULL,
    ran_at         DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_reconciliations_batch FOREIGN KEY (batch_id) REFERENCES batches (id)
);
