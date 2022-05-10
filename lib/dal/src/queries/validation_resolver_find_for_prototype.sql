SELECT DISTINCT ON (validation_resolvers.id) validation_resolvers.id,
                              validation_resolvers.prop_id,
                              validation_resolvers.visibility_change_set_pk,
                              validation_resolvers.visibility_edit_session_pk,
                              validation_resolvers.component_id,
                              validation_resolvers.schema_id,
                              validation_resolvers.schema_variant_id,
                              validation_resolvers.system_id,
                              row_to_json(validation_resolvers.*) as object
FROM validation_resolvers
WHERE in_tenancy_v1($1, validation_resolvers.tenancy_universal, validation_resolvers.tenancy_billing_account_ids, validation_resolvers.tenancy_organization_ids,
                    validation_resolvers.tenancy_workspace_ids)
  AND is_visible_v1($2, validation_resolvers.visibility_change_set_pk, validation_resolvers.visibility_edit_session_pk, validation_resolvers.visibility_deleted_at)
  AND validation_resolvers.validation_prototype_id = $3
	ORDER BY validation_resolvers.id, 
      visibility_change_set_pk DESC, 
      visibility_edit_session_pk DESC, 
      prop_id DESC,
      component_id DESC, 
      system_id DESC, 
      schema_variant_id DESC, 
      schema_id DESC;

