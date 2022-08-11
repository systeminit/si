SELECT DISTINCT ON (resource_resolvers.id) resource_resolvers.id,
                                           resource_resolvers.visibility_change_set_pk,

                                           row_to_json(func_binding_return_values.*) AS object
FROM resource_resolvers
         INNER JOIN func_binding_return_value_belongs_to_func_binding ON
        func_binding_return_value_belongs_to_func_binding.belongs_to_id = resource_resolvers.func_binding_id
         INNER JOIN func_binding_return_values ON
        func_binding_return_values.id = func_binding_return_value_belongs_to_func_binding.object_id
WHERE in_tenancy_v1($1, resource_resolvers.tenancy_universal, resource_resolvers.tenancy_billing_account_ids,
                    resource_resolvers.tenancy_organization_ids,
                    resource_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, resource_resolvers.visibility_change_set_pk, resource_resolvers.visibility_deleted_at)
  AND is_visible_v1($2,
                    func_binding_return_value_belongs_to_func_binding.visibility_change_set_pk,
                    func_binding_return_value_belongs_to_func_binding.visibility_deleted_at)
  AND is_visible_v1($2,
                    func_binding_return_values.visibility_change_set_pk,
                    func_binding_return_values.visibility_deleted_at)
  AND resource_resolvers.component_id = $3
  AND (resource_resolvers.system_id = $4 OR resource_resolvers.system_id = -1)
ORDER BY resource_resolvers.id DESC,
         visibility_change_set_pk DESC,
         func_binding_return_values.id DESC
LIMIT 1;
