SELECT row_to_json(wspaces.*) AS object FROM (

(
    SELECT w.*
    FROM workspaces AS w 
    WHERE w.pk = $1
    ORDER BY w.created_at DESC
)

UNION

(
    SELECT w.*
    FROM workspaces AS w 
        JOIN change_set_pointers AS csp ON csp.workspace_id = w.pk
    WHERE 
        csp.id = $1
    ORDER BY w.created_at DESC
)

UNION 

(
    SELECT w.* AS object 
    FROM workspaces AS w 
        JOIN user_belongs_to_workspaces AS ubtw ON ubtw.workspace_pk = w.pk
        JOIN users AS u ON u.pk = ubtw.user_pk
    WHERE u.pk = $1
    ORDER BY w.created_at DESC
)

) AS wspaces

ORDER BY wspaces.created_at DESC
