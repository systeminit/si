SELECT row_to_json(organizations.*) AS organization,
       row_to_json(workspaces.*)    AS workspace
FROM organizations
INNER JOIN workspace_belongs_to_organization
    ON workspace_belongs_to_organization.belongs_to_id = organizations.id
       AND workspace_belongs_to_organization.visibility_deleted_at IS NULL
INNER JOIN workspaces
    ON workspaces.id = workspace_belongs_to_organization.object_id
       AND workspaces.visibility_deleted_at IS NULL
WHERE organizations.billing_account_pk = $1
  AND organizations.name = 'default'
  AND organizations.visibility_deleted_at IS NULL
ORDER BY organizations.id, workspaces.id
LIMIT 1;
