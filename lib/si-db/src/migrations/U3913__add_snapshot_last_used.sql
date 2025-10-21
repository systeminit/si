-- Create tracking table for snapshot eviction
CREATE TABLE IF NOT EXISTS snapshot_last_used (
    snapshot_id TEXT PRIMARY KEY,
    last_used_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CLOCK_TIMESTAMP()
);

-- Index for efficient candidate queries
CREATE INDEX IF NOT EXISTS idx_snapshot_last_used_at
ON snapshot_last_used(last_used_at);

-- Note: Index on change_set_pointers.workspace_snapshot_address
-- already exists as change_set_pointers_snapshot_idx from U3200
