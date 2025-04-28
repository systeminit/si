ALTER TABLE workspaces ADD COLUMN snapshot_version text NOT NULL DEFAULT 'V1';
-- When this migration is run, we have not yet migrated our snapshots
UPDATE workspaces SET snapshot_version = 'Legacy';