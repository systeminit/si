SELECT DISTINCT ON (component_id) component_id,
                                  components.prop_values -> 'si' ->> 'name' AS component_name

FROM components_with_attributes AS components

-- Ensure they are deleted
WHERE visibility_deleted_at IS NOT NULL

  -- Compare only to the current change set
  AND visibility_change_set_pk = $2

  -- Scope the tenancy one last time
  AND in_tenancy_v1($1,
                    tenancy_billing_account_pks,
                    tenancy_organization_ids,
                    tenancy_workspace_ids)

ORDER BY component_id DESC,
         component_name DESC
