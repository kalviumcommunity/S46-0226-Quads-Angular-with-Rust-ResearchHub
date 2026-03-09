CREATE TABLE IF NOT EXISTS research_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    research_item_id UUID NOT NULL REFERENCES research_items(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    type research_item_type NOT NULL,
    visibility research_visibility NOT NULL,
    file_url TEXT,
    file_name TEXT,
    file_size_bytes BIGINT,
    mime_type TEXT,
    file_checksum TEXT,
    changed_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (research_item_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_research_versions_item ON research_versions(research_item_id);
CREATE INDEX IF NOT EXISTS idx_research_versions_changed_by ON research_versions(changed_by);
