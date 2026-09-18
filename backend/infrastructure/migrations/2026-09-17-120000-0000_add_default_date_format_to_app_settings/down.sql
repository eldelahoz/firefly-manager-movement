-- Remove default_date_format from app_settings (RF-02 v0.8, RF-19)

ALTER TABLE app_settings DROP COLUMN default_date_format
