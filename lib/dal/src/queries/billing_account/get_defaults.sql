-- TODO: Not quite sure what's going on here with the pk = pk matching. Older understanding of how visibility querying works?
SELECT row_to_json(organizations.*)                                     AS organization,
       row_to_json(workspaces.*)                                        AS workspace,
       organization_belongs_to_billing_account.id                       AS id,
       organization_belongs_to_billing_account.visibility_change_set_pk AS visibility_change_set_pk
FROM organization_belongs_to_billing_account
         INNER JOIN workspace_belongs_to_organization
                    ON workspace_belongs_to_organization.belongs_to_id =
                       organization_belongs_to_billing_account.object_id
                        AND organization_belongs_to_billing_account.visibility_change_set_pk =
                            workspace_belongs_to_organization.visibility_change_set_pk
                        AND workspace_belongs_to_organization.visibility_deleted_at IS NULL
         INNER JOIN workspaces ON workspace_belongs_to_organization.object_id = workspaces.id
    AND workspaces.name = 'default'
    AND organization_belongs_to_billing_account.visibility_change_set_pk = workspaces.visibility_change_set_pk
    AND workspaces.visibility_deleted_at IS NULL
         INNER JOIN organizations ON organization_belongs_to_billing_account.object_id = organizations.id
    AND organizations.name = 'default'
    AND organization_belongs_to_billing_account.visibility_change_set_pk = organizations.visibility_change_set_pk
    AND organizations.visibility_deleted_at IS NULL
WHERE organization_belongs_to_billing_account.belongs_to_id = $3
  AND in_tenancy_v1($1, organization_belongs_to_billing_account.tenancy_universal,
                    organization_belongs_to_billing_account.tenancy_billing_account_ids,
                    organization_belongs_to_billing_account.tenancy_organization_ids,
                    organization_belongs_to_billing_account.tenancy_workspace_ids)
  AND is_visible_v1($2, organization_belongs_to_billing_account.visibility_change_set_pk,
                    organization_belongs_to_billing_account.visibility_deleted_at)
ORDER BY id, visibility_change_set_pk DESC
LIMIT 1;
