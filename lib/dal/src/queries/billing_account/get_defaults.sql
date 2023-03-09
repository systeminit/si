SELECT row_to_json(workspaces.*)    AS workspace
FROM workspaces
WHERE workspaces.billing_account_pk = $1
  AND workspaces.visibility_deleted_at IS NULL
ORDER BY workspaces.pk
LIMIT 1;
