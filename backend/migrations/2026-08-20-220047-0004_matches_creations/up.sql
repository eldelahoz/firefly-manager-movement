-- matches: emparejamientos 1-a-1 historizados por run (el 1-a-1 lo garantiza el schema).
-- creations: trazabilidad de tx creadas por nosotros (UNIQUE(movement_id) = idempotencia NFR-05).

CREATE TABLE matches (
    id                  BIGINT   NOT NULL AUTO_INCREMENT PRIMARY KEY,
    reconciliation_id   BIGINT   NOT NULL,
    movement_id         BIGINT   NOT NULL,
    ff3_transaction_id  BIGINT   NOT NULL,
    matched_at          DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_matches_reconciliation FOREIGN KEY (reconciliation_id) REFERENCES reconciliations (id),
    CONSTRAINT fk_matches_movement FOREIGN KEY (movement_id) REFERENCES movements (id),
    CONSTRAINT fk_matches_ff3_transaction FOREIGN KEY (ff3_transaction_id) REFERENCES ff3_transactions (id),
    UNIQUE KEY uq_matches_run_movement (reconciliation_id, movement_id),
    UNIQUE KEY uq_matches_run_ff3 (reconciliation_id, ff3_transaction_id)
);

CREATE TABLE creations (
    id          BIGINT   NOT NULL AUTO_INCREMENT PRIMARY KEY,
    movement_id BIGINT   NOT NULL,
    ff3_id      BIGINT   NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_creations_movement FOREIGN KEY (movement_id) REFERENCES movements (id),
    UNIQUE KEY uq_creations_movement (movement_id)
);
