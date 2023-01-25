SELECT DISTINCT ON (workspaces.id) workspaces.id                    as workspace_id,
                                   organizations.pk                 as organization_pk,
                                   organizations.billing_account_pk as billing_account_pk
FROM workspaces
INNER JOIN organizations
    ON organizations.pk = workspaces.organization_pk
       AND organizations.visibility_deleted_at IS NULL
WHERE workspaces.id = $1
      AND is_visible_v1($2, workspaces.visibility_change_set_pk, workspaces.visibility_deleted_at)
ORDER BY workspaces.id DESC,
         workspaces.visibility_change_set_pk DESC
LIMIT 1;
