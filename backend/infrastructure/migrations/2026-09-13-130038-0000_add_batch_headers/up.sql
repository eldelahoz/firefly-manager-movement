-- batches: persist the CSV headers and detected delimiter so an unfinished
-- import can be resumed (RF-18) with its original mapping columns (RF-02).

ALTER TABLE batches
    ADD COLUMN headers_json JSON NOT NULL DEFAULT ('[]'),
    ADD COLUMN detected_delimiter VARCHAR(8) NOT NULL DEFAULT ',';
