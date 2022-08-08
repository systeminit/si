SELECT DISTINCT ON (components.id) components.visibility_change_set_pk,

                                   components.visibility_deleted_at,
                                   row_to_json(components.*) AS object
FROM components
         INNER JOIN component_belongs_to_schema_variant bt ON bt.object_id = components.id
    AND is_visible_v1(
                                                                      $1,
                                                                      bt.visibility_change_set_pk,
                                                                      bt.visibility_deleted_at
                                                                  )
    AND in_tenancy_v1(
                                                                      $2,
                                                                      bt.tenancy_universal,
                                                                      bt.tenancy_billing_account_ids,
                                                                      bt.tenancy_organization_ids,
                                                                      bt.tenancy_workspace_ids)
WHERE is_visible_v1(
        $1,
        components.visibility_change_set_pk,
        components.visibility_deleted_at
    )
  AND in_tenancy_v1(
        $2,
        components.tenancy_universal,
        components.tenancy_billing_account_ids,
        components.tenancy_organization_ids,
        components.tenancy_workspace_ids)
  AND bt.belongs_to_id = $3
ORDER BY components.id,
         components.visibility_change_set_pk DESC,
         components.visibility_deleted_at DESC NULLS FIRST;
