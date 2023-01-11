SELECT DISTINCT ON (workspaces.id) workspaces.id                                         as workspace_id,
                                   workspace_belongs_to_organization.belongs_to_id       as organization_id,
                                   organization_belongs_to_billing_account.belongs_to_id as billing_account_id
FROM workspaces
         INNER JOIN workspace_belongs_to_organization ON workspace_belongs_to_organization.object_id = workspaces.id
    AND is_visible_v1(
                                                                 $2,
                                                                 workspace_belongs_to_organization.visibility_change_set_pk,
                                                                 workspace_belongs_to_organization.visibility_deleted_at)
         INNER JOIN organization_belongs_to_billing_account on organization_belongs_to_billing_account.object_id =
                                                               workspace_belongs_to_organization.belongs_to_id
    AND is_visible_v1(
                                                                       $2,
                                                                       organization_belongs_to_billing_account.visibility_change_set_pk,
                                                                       organization_belongs_to_billing_account.visibility_deleted_at)

WHERE workspaces.id = $1
  AND is_visible_v1($2, workspaces.visibility_change_set_pk, workspaces.visibility_deleted_at)
ORDER BY workspaces.id DESC,
         workspaces.visibility_change_set_pk DESC
LIMIT 1;
