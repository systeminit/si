-- Migration: Populate snapshot_last_used for existing snapshots
--
-- This migration is safe to run multiple times (ON CONFLICT DO NOTHING).
--
-- Old, unused snapshots (not referenced by any change set) are NOT tracked.
-- These will be handled by future general GC work (out of scope).
--
-- Snapshots created after this migration will be tracked automatically when
-- change sets move away from them (via ChangeSet::update_pointer).

-- Populate snapshot_last_used for all currently-used snapshots
INSERT INTO snapshot_last_used (snapshot_id, last_used_at, created_at)
SELECT DISTINCT
    workspace_snapshot_address as snapshot_id,
    CLOCK_TIMESTAMP() as last_used_at,
    CLOCK_TIMESTAMP() as created_at
FROM change_set_pointers
WHERE workspace_snapshot_address IS NOT NULL
ON CONFLICT (snapshot_id) DO NOTHING;

-- Log how many snapshots were tracked
DO $$
DECLARE
    snapshot_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO snapshot_count FROM snapshot_last_used;
    RAISE NOTICE 'Populated snapshot_last_used with % snapshots', snapshot_count;
END $$;
