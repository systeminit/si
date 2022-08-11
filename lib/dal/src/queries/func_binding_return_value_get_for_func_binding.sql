SELECT DISTINCT ON (func_binding_return_values.id) func_binding_return_values.id,
                                                   func_binding_return_values.visibility_change_set_pk,

                                                   func_binding_return_values.unprocessed_value,
                                                   func_binding_return_values.value,
                                                   row_to_json(func_binding_return_values.*) as object
FROM func_binding_return_values
         INNER JOIN func_binding_return_value_belongs_to_func_binding ON
        func_binding_return_value_belongs_to_func_binding.object_id = func_binding_return_values.id
WHERE in_tenancy_v1($1, func_binding_return_values.tenancy_universal,
                    func_binding_return_values.tenancy_billing_account_ids,
                    func_binding_return_values.tenancy_organization_ids,
                    func_binding_return_values.tenancy_workspace_ids)
  AND is_visible_v1($2, func_binding_return_values.visibility_change_set_pk,
                    func_binding_return_values.visibility_deleted_at)
  AND is_visible_v1($2,
                    func_binding_return_value_belongs_to_func_binding.visibility_change_set_pk,
                    func_binding_return_value_belongs_to_func_binding.visibility_deleted_at)
  AND func_binding_return_value_belongs_to_func_binding.belongs_to_id = $3
ORDER BY func_binding_return_values.id,
         visibility_change_set_pk DESC,
         func_binding_return_value_belongs_to_func_binding.belongs_to_id DESC,
         func_binding_return_value_belongs_to_func_binding.object_id DESC
LIMIT 1;
