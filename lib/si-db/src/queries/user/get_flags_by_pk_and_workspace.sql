SELECT u.flag_map AS object
FROM user_belongs_to_workspaces u
WHERE u.user_pk = $1
  AND u.workspace_pk = $2
  AND u.visibility_deleted_at IS NULL
