SELECT row_to_json(users.*) AS object
FROM users
INNER JOIN user_belongs_to_workspaces bt
  ON bt.user_pk = users.pk
     AND bt.visibility_deleted_at IS NULL
WHERE users.email = $1
      AND users.visibility_deleted_at IS NULL
      AND bt.workspace_pk = $2
