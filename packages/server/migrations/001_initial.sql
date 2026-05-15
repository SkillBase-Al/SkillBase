-- pgvector extension is optional for MVP (TF-IDF used instead)
-- To enable vector search: CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(512) NOT NULL,
    description TEXT,
    source VARCHAR(64) NOT NULL,
    source_url TEXT,
    license VARCHAR(128),
    content_hash VARCHAR(64) NOT NULL UNIQUE,
    skill_md_content TEXT,
    safety_level VARCHAR(32),
    format_score INTEGER,
    quality_score INTEGER,
    rating DOUBLE PRECISION,
    install_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(128) NOT NULL UNIQUE,
    display_name VARCHAR(256) NOT NULL
);

CREATE TABLE IF NOT EXISTS skill_categories (
    skill_id UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (skill_id, category_id)
);

CREATE INDEX IF NOT EXISTS idx_skills_name ON skills(name);
CREATE INDEX IF NOT EXISTS idx_skills_source ON skills(source);
CREATE INDEX IF NOT EXISTS idx_skills_license ON skills(license);
CREATE INDEX IF NOT EXISTS idx_skills_rating ON skills(rating DESC);
CREATE INDEX IF NOT EXISTS idx_skills_install_count ON skills(install_count DESC);
CREATE INDEX IF NOT EXISTS idx_skills_content_hash ON skills(content_hash);
CREATE INDEX IF NOT EXISTS idx_skill_categories_category_id ON skill_categories(category_id);
CREATE INDEX IF NOT EXISTS idx_skill_categories_skill_id ON skill_categories(skill_id);
