SELECT DISTINCT ON (attribute_resolvers.prop_id) attribute_resolvers.id,
                              attribute_resolvers.prop_id,
                              attribute_resolvers.visibility_change_set_pk,
                              attribute_resolvers.visibility_edit_session_pk,
                              attribute_resolvers.component_id,
                              attribute_resolvers.schema_id,
                              attribute_resolvers.schema_variant_id,
                              attribute_resolvers.system_id,
                              row_to_json(attribute_resolvers.*) AS object
FROM attribute_resolvers
WHERE in_tenancy_v1($1, attribute_resolvers.tenancy_universal, attribute_resolvers.tenancy_billing_account_ids, attribute_resolvers.tenancy_organization_ids,
                    attribute_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_resolvers.visibility_change_set_pk, attribute_resolvers.visibility_edit_session_pk, attribute_resolvers.visibility_deleted)
  AND attribute_resolvers.prop_id = $3
  AND (attribute_resolvers.component_id = $4 OR attribute_resolvers.component_id = -1)
  AND (attribute_resolvers.schema_id = $5 OR attribute_resolvers.schema_id = -1)
  AND (attribute_resolvers.schema_variant_id = $6 OR attribute_resolvers.schema_variant_id = -1)
  AND (attribute_resolvers.system_id = $7 OR attribute_resolvers.system_id = -1)
	ORDER BY prop_id, 
      visibility_change_set_pk DESC, 
      visibility_edit_session_pk DESC, 
      component_id DESC, 
      system_id DESC, 
      schema_variant_id DESC, 
      schema_id DESC
  LIMIT 1;
