SELECT row_to_json(u.*) AS object
FROM users AS u
INNER JOIN user_belongs_to_workspaces bt ON bt.user_pk = u.pk
WHERE bt.workspace_pk = $1
ORDER BY u.created_at ASC
