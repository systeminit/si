SELECT row_to_json(w.*) AS object
FROM workspaces AS w
WHERE name = $1 AND visibility_deleted_at is NULL
