SELECT row_to_json(w.*) AS object 
    FROM workspaces AS w 
        JOIN change_set_pointers AS csp ON csp.workspace_id = w.pk
    WHERE 
        csp.workspace_snapshot_address = $1
    ORDER BY w.created_at DESC
