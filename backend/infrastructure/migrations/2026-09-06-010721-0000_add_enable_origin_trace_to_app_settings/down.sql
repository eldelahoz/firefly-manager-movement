-- Remove enable_origin_trace from app_settings (RF-11 v0.8, RF-19 v0.8)

ALTER TABLE app_settings DROP COLUMN enable_origin_trace
