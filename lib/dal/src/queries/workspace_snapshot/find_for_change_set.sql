SELECT * FROM workspace_snapshots
JOIN change_set_pointers
    ON change_set_pointers.id = $1
           AND change_set_pointers.workspace_snapshot_id = workspace_snapshots.id