ALTER TABLE research_items
    ADD COLUMN IF NOT EXISTS file_name TEXT,
    ADD COLUMN IF NOT EXISTS file_size_bytes BIGINT,
    ADD COLUMN IF NOT EXISTS mime_type TEXT,
    ADD COLUMN IF NOT EXISTS file_checksum TEXT;

CREATE INDEX IF NOT EXISTS idx_research_file_checksum ON research_items(file_checksum);
