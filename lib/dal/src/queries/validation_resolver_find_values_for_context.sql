SELECT DISTINCT ON (validation_resolvers.id) validation_resolvers.id,
                              validation_resolvers.prop_id,
                              validation_resolvers.visibility_change_set_pk,
                              validation_resolvers.visibility_edit_session_pk,
                              validation_resolvers.component_id,
                              validation_resolvers.schema_id,
                              validation_resolvers.schema_variant_id,
                              validation_resolvers.system_id,
                              validation_resolvers.validation_prototype_id,
                              row_to_json(func_binding_return_values.*) AS object
FROM validation_resolvers
INNER JOIN func_binding_return_value_belongs_to_func_binding ON 
  func_binding_return_value_belongs_to_func_binding.belongs_to_id = validation_resolvers.func_binding_id
INNER JOIN func_binding_return_values ON 
  func_binding_return_values.id = func_binding_return_value_belongs_to_func_binding.object_id
WHERE in_tenancy_v1($1, validation_resolvers.tenancy_universal, validation_resolvers.tenancy_billing_account_ids, validation_resolvers.tenancy_organization_ids,
                    validation_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, validation_resolvers.visibility_change_set_pk, validation_resolvers.visibility_edit_session_pk, validation_resolvers.visibility_deleted)
  AND is_visible_v1($2, 
    func_binding_return_value_belongs_to_func_binding.visibility_change_set_pk, 
    func_binding_return_value_belongs_to_func_binding.visibility_edit_session_pk, 
    func_binding_return_value_belongs_to_func_binding.visibility_deleted)
  AND is_visible_v1($2, 
    func_binding_return_values.visibility_change_set_pk, 
    func_binding_return_values.visibility_edit_session_pk, 
    func_binding_return_values.visibility_deleted)
  AND validation_resolvers.prop_id = $3
   AND validation_resolvers.component_id = $4
   AND (validation_resolvers.system_id = $5 OR validation_resolvers.system_id = -1)
	ORDER BY validation_resolvers.id, 
      visibility_change_set_pk DESC, 
      visibility_edit_session_pk DESC, 
      prop_id DESC,
      component_id DESC, 
      system_id DESC, 
      schema_variant_id DESC, 
      schema_id DESC;

