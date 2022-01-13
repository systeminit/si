SELECT DISTINCT ON (attribute_resolvers.prop_id) attribute_resolvers.id,
                              attribute_resolvers.prop_id,
                              attribute_resolvers.visibility_change_set_pk,
                              attribute_resolvers.visibility_edit_session_pk,
                              attribute_resolvers.component_id,
                              attribute_resolvers.schema_id,
                              attribute_resolvers.schema_variant_id,
                              attribute_resolvers.system_id,
                              row_to_json(func_binding_return_values.*) AS object
FROM attribute_resolvers
INNER JOIN func_binding_return_value_belongs_to_func_binding ON 
  func_binding_return_value_belongs_to_func_binding.belongs_to_id = attribute_resolvers.func_binding_id
INNER JOIN func_binding_return_values ON 
  func_binding_return_values.id = func_binding_return_value_belongs_to_func_binding.object_id
WHERE in_tenancy_v1($1, attribute_resolvers.tenancy_universal, attribute_resolvers.tenancy_billing_account_ids, attribute_resolvers.tenancy_organization_ids,
                    attribute_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_resolvers.visibility_change_set_pk, attribute_resolvers.visibility_edit_session_pk, attribute_resolvers.visibility_deleted)
  AND is_visible_v1($2, 
    func_binding_return_value_belongs_to_func_binding.visibility_change_set_pk, 
    func_binding_return_value_belongs_to_func_binding.visibility_edit_session_pk, 
    func_binding_return_value_belongs_to_func_binding.visibility_deleted)
  AND is_visible_v1($2, 
    func_binding_return_values.visibility_change_set_pk, 
    func_binding_return_values.visibility_edit_session_pk, 
    func_binding_return_values.visibility_deleted)
  AND attribute_resolvers.prop_id = $3
   AND (attribute_resolvers.component_id = $4 OR attribute_resolvers.component_id = -1)
   AND (attribute_resolvers.system_id = $5 OR attribute_resolvers.system_id = -1)
	ORDER BY prop_id, 
      visibility_change_set_pk DESC, 
      visibility_edit_session_pk DESC, 
      component_id DESC, 
      system_id DESC, 
      schema_variant_id DESC, 
      schema_id DESC
  LIMIT 1;
