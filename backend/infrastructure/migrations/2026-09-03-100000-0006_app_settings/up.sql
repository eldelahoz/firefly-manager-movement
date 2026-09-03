-- App settings (RF-19): single row (id = 1). ff3_token is write-only (NFR-02);
-- default_creation_tag nullable (empty = no default tag, RF-19 v0.7).

CREATE TABLE app_settings (
    id INT NOT NULL PRIMARY KEY,
    ff3_url VARCHAR(255) NOT NULL,
    ff3_token VARCHAR(255) NOT NULL,
    default_creation_tag VARCHAR(255) NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
)
