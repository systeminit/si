SELECT row_to_json(organizations.*) AS organization,
       row_to_json(workspaces.*)    AS workspace
FROM organizations
INNER JOIN workspaces
    ON workspaces.organization_pk = organizations.pk
       AND workspaces.visibility_deleted_at IS NULL
WHERE organizations.billing_account_pk = $1
  AND organizations.name = 'default'
  AND organizations.visibility_deleted_at IS NULL
ORDER BY organizations.pk, workspaces.id
LIMIT 1;
