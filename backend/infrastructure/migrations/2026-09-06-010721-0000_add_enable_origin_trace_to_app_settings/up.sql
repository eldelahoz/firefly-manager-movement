-- Add enable_origin_trace to app_settings (RF-11 v0.8, RF-19 v0.8)
-- Default: true (origin trace enabled by default)

ALTER TABLE app_settings ADD COLUMN enable_origin_trace BOOLEAN NOT NULL DEFAULT TRUE
