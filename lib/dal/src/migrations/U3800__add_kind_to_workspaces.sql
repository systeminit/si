ALTER TABLE workspaces
    ADD COLUMN snapshot_kind text DEFAULT 'LegacySnapshot';
