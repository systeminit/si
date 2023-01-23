SELECT DISTINCT ON (component_id) component_id,
                                  components.prop_values -> 'si' ->> 'name' AS component_name

FROM components_with_attributes AS components

-- Find components that are not in HEAD
WHERE component_id NOT IN (SELECT id
                           FROM components
                           WHERE visibility_change_set_pk = ident_nil_v1()
                             AND visibility_deleted_at IS NULL
                             AND in_tenancy_v1($1,
                                               tenancy_billing_account_pks,
                                               tenancy_organization_ids,
                                               tenancy_workspace_ids))

  -- Compare only to the current change set
  AND visibility_change_set_pk = $2

  -- Ensure they are not deleted
  AND visibility_deleted_at IS NULL

  -- Scope the tenancy one last time
  AND in_tenancy_v1($1,
                    tenancy_billing_account_pks,
                    tenancy_organization_ids,
                    tenancy_workspace_ids)

ORDER BY component_id DESC,
         component_name DESC
