SELECT DISTINCT ON (code_generation_resolvers.id) code_generation_resolvers.id,
                              code_generation_resolvers.visibility_change_set_pk,
                              code_generation_resolvers.visibility_edit_session_pk,
                              code_generation_prototypes.format AS format,
                              row_to_json(func_binding_return_values.*) AS object
FROM code_generation_resolvers
INNER JOIN func_binding_return_value_belongs_to_func_binding ON 
  func_binding_return_value_belongs_to_func_binding.belongs_to_id = code_generation_resolvers.func_binding_id
INNER JOIN func_binding_return_values ON 
  func_binding_return_values.id = func_binding_return_value_belongs_to_func_binding.object_id
INNER JOIN code_generation_prototypes ON
  code_generation_prototypes.id = code_generation_resolvers.code_generation_prototype_id
WHERE in_tenancy_v1($1, code_generation_resolvers.tenancy_universal, code_generation_resolvers.tenancy_billing_account_ids, code_generation_resolvers.tenancy_organization_ids,
                    code_generation_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, code_generation_resolvers.visibility_change_set_pk, code_generation_resolvers.visibility_edit_session_pk, code_generation_resolvers.visibility_deleted_at)
  AND is_visible_v1($2, 
    func_binding_return_value_belongs_to_func_binding.visibility_change_set_pk, 
    func_binding_return_value_belongs_to_func_binding.visibility_edit_session_pk, 
    func_binding_return_value_belongs_to_func_binding.visibility_deleted_at)
  AND is_visible_v1($2, 
    func_binding_return_values.visibility_change_set_pk, 
    func_binding_return_values.visibility_edit_session_pk, 
    func_binding_return_values.visibility_deleted_at)
  AND code_generation_resolvers.component_id = $3
   AND (code_generation_resolvers.system_id = $4 OR code_generation_resolvers.system_id = -1)
	ORDER BY code_generation_resolvers.id, 
      visibility_change_set_pk DESC, 
      visibility_edit_session_pk DESC;
