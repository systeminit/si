CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS change_set_pointers_snapshot_idx ON change_set_pointers (workspace_snapshot_address);
CREATE INDEX IF NOT EXISTS change_set_pointers_workspace_id_idx ON change_set_pointers (workspace_id);
CREATE INDEX IF NOT EXISTS users_name_email_idx ON users USING gin ((name || ' ' || email) gin_trgm_ops);