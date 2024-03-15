SELECT *
FROM workspace_snapshots
         JOIN change_sets
              ON change_sets.id = $1
                  AND change_sets.workspace_snapshot_id = workspace_snapshots.id