SELECT row_to_json(w.*) AS object 
    FROM workspaces AS w 
        JOIN user_belongs_to_workspaces AS ubtw ON ubtw.workspace_pk = w.pk
        JOIN users AS u ON u.pk = ubtw.user_pk
    WHERE u.name || ' ' || u.email ILIKE $1
    ORDER BY w.created_at DESC
