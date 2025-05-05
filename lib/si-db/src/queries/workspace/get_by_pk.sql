SELECT * FROM workspaces AS w
WHERE pk = $1 AND visibility_deleted_at is NULL
