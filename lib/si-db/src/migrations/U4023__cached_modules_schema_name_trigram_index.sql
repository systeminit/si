-- Enable the pg_trgm extension for trigram-based fuzzy text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Add a GIN trigram index on schema_name for fast fuzzy/ILIKE searches
CREATE INDEX IF NOT EXISTS cached_modules_schema_name_trgm_idx
ON cached_modules USING gin (schema_name gin_trgm_ops);

-- Add a GIN index for full-text search on schema_name
CREATE INDEX IF NOT EXISTS cached_modules_schema_name_fts_idx
ON cached_modules USING gin (to_tsvector('english', schema_name));
