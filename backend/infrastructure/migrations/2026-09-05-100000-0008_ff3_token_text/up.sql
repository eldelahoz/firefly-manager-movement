-- RF-19 fix: Firefly III OAuth personal access tokens are ~1 KB JWTs;
-- VARCHAR(255) rejects them with MySQL error 1406 (Data too long).

ALTER TABLE app_settings MODIFY ff3_token TEXT NOT NULL
