DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'research_item_type') THEN
        CREATE TYPE research_item_type AS ENUM ('paper', 'dataset', 'code');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'research_visibility') THEN
        CREATE TYPE research_visibility AS ENUM ('private', 'group', 'institution', 'public');
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS research_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    type research_item_type NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    version INTEGER NOT NULL DEFAULT 1,
    visibility research_visibility NOT NULL DEFAULT 'private',
    file_url TEXT,
    institution_id UUID,
    group_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_owner ON research_items(owner_id);
CREATE INDEX IF NOT EXISTS idx_research_type ON research_items(type);
CREATE INDEX IF NOT EXISTS idx_research_visibility ON research_items(visibility);
