-- Add default_date_format to app_settings (RF-02 v0.8, RF-19)
-- NULL = no global default (parser falls back to legacy auto-try)

ALTER TABLE app_settings ADD COLUMN default_date_format VARCHAR(16) NULL
