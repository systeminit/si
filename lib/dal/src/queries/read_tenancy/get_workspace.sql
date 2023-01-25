SELECT DISTINCT ON (workspaces.id) workspaces.id                                   as workspace_id,
                                   workspace_belongs_to_organization.belongs_to_id as organization_id,
                                   organizations.billing_account_pk                as billing_account_pk
FROM workspaces
INNER JOIN workspace_belongs_to_organization ON workspace_belongs_to_organization.object_id = workspaces.id
    AND is_visible_v1($2,
                      workspace_belongs_to_organization.visibility_change_set_pk,
                      workspace_belongs_to_organization.visibility_deleted_at)
INNER JOIN organizations
    ON organizations.id = workspace_belongs_to_organization.belongs_to_id
       AND is_visible_v1($2,
                         organizations.visibility_change_set_pk,
                         organizations.visibility_deleted_at)

WHERE workspaces.id = $1
      AND is_visible_v1($2, workspaces.visibility_change_set_pk, workspaces.visibility_deleted_at)
ORDER BY workspaces.id DESC,
         workspaces.visibility_change_set_pk DESC
LIMIT 1;
