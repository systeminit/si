SELECT DISTINCT ON (attribute_resolvers.id) attribute_resolvers.id,
                              attribute_resolvers.prop_id,
                              attribute_resolvers.visibility_change_set_pk,
                              attribute_resolvers.visibility_edit_session_pk,
                              attribute_resolvers.component_id,
                              attribute_resolvers.schema_id,
                              attribute_resolvers.schema_variant_id,
                              attribute_resolvers.system_id,
                              prop_belongs_to_prop.belongs_to_id AS parent_prop_id,
                              attribute_resolver_belongs_to_attribute_resolver.belongs_to_id AS parent_attribute_resolver_id,
                              row_to_json(attribute_resolvers.*) AS attribute_resolver_object,
                              row_to_json(props.*) AS prop_object,
                              row_to_json(func_binding_return_values.*) AS object
FROM attribute_resolvers
-- First, we need to extract the schema_variant_id for this component
INNER JOIN component_belongs_to_schema_variant ON
    component_belongs_to_schema_variant.object_id = $3
-- Second, we need to join on all the props that are relevant for that schema variant
INNER JOIN props ON props.id IN (
    WITH RECURSIVE recursive_props AS (
        SELECT left_object_id AS prop_id
        FROM prop_many_to_many_schema_variants
        WHERE right_object_id = component_belongs_to_schema_variant.belongs_to_id
        UNION ALL
        SELECT pbp.object_id AS prop_id
        FROM prop_belongs_to_prop AS pbp
          JOIN recursive_props ON pbp.belongs_to_id = recursive_props.prop_id
    )
    SELECT prop_id
    FROM recursive_props
) AND props.id = attribute_resolvers.prop_id
-- Third, we need to find all the return values
INNER JOIN func_binding_return_value_belongs_to_func_binding ON
        func_binding_return_value_belongs_to_func_binding.belongs_to_id = attribute_resolvers.func_binding_id
INNER JOIN func_binding_return_values ON
  func_binding_return_values.id = func_binding_return_value_belongs_to_func_binding.object_id
LEFT JOIN prop_belongs_to_prop ON props.id = prop_belongs_to_prop.object_id
LEFT JOIN attribute_resolver_belongs_to_attribute_resolver ON attribute_resolvers.id = attribute_resolver_belongs_to_attribute_resolver.object_id
WHERE in_tenancy_v1($1, attribute_resolvers.tenancy_universal, attribute_resolvers.tenancy_billing_account_ids, attribute_resolvers.tenancy_organization_ids,
                    attribute_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_resolvers.visibility_change_set_pk, attribute_resolvers.visibility_edit_session_pk, attribute_resolvers.visibility_deleted)
  AND is_visible_v1($2, props.visibility_change_set_pk, props.visibility_edit_session_pk, props.visibility_deleted)
  AND is_visible_v1($2, 
    func_binding_return_value_belongs_to_func_binding.visibility_change_set_pk,
    func_binding_return_value_belongs_to_func_binding.visibility_edit_session_pk,
    func_binding_return_value_belongs_to_func_binding.visibility_deleted)
  AND is_visible_v1($2, 
    func_binding_return_values.visibility_change_set_pk, 
    func_binding_return_values.visibility_edit_session_pk, 
    func_binding_return_values.visibility_deleted)
  AND (attribute_resolvers.component_id = $3 OR attribute_resolvers.component_id = -1)
  AND (attribute_resolvers.system_id = $4 OR attribute_resolvers.system_id = -1)
	ORDER BY 
      attribute_resolvers.id,
      prop_id, 
      visibility_change_set_pk DESC, 
      visibility_edit_session_pk DESC, 
      parent_prop_id DESC,
      component_id DESC, 
      system_id DESC, 
      schema_variant_id DESC, 
      schema_id DESC;
