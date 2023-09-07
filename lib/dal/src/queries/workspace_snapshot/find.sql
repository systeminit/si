SELECT row_to_json(workspace_snapshots.*) AS object
FROM workspace_snapshots
    WHERE workspace_snapshots.id = $1
