CREATE INDEX idx_modules_kind_rejected_schema_created
ON modules (kind, rejected_at, schema_id, created_at DESC)
WHERE schema_id IS NOT NULL AND is_builtin_at IS NOT NULL;
