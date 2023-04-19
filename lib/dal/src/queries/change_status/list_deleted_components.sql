SELECT DISTINCT ON (component_id) component_id,
                                  components.prop_values -> 'si' ->> 'name' AS component_name

FROM components_with_attributes_v2 AS components

-- Ensure they are deleted
WHERE visibility_deleted_at IS NOT NULL

  -- Compare only to the current change set
  AND visibility_change_set_pk = $2

  -- Scope the tenancy one last time
  AND in_tenancy_v1($1, tenancy_workspace_pk)

ORDER BY component_id DESC,
         component_name DESC
