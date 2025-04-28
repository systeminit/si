SELECT row_to_json(w.*) AS object
FROM workspaces AS w
INNER JOIN user_belongs_to_workspaces bt ON bt.workspace_pk = w.pk
WHERE bt.user_pk = $1
ORDER BY w.created_at ASC
