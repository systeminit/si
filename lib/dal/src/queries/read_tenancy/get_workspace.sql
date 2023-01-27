SELECT DISTINCT ON (workspaces.pk) workspaces.pk                    as workspace_pk,
                                   organizations.pk                 as organization_pk,
                                   organizations.billing_account_pk as billing_account_pk
FROM workspaces
INNER JOIN organizations
    ON organizations.pk = workspaces.organization_pk
       AND organizations.visibility_deleted_at IS NULL
WHERE workspaces.pk = $1
      AND workspaces.visibility_deleted_at IS NULL
ORDER BY workspaces.pk DESC
LIMIT 1;
