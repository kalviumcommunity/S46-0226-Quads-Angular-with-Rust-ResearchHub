ALTER TABLE research_items
    ADD COLUMN IF NOT EXISTS doi TEXT,
    ADD COLUMN IF NOT EXISTS citation_authors TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS citation_year INTEGER;

CREATE INDEX IF NOT EXISTS idx_research_doi ON research_items(doi);
