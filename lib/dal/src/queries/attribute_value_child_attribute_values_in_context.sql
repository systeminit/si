SELECT DISTINCT ON (attribute_values.attribute_context_prop_id, attribute_values.key)
                   attribute_values.id,
                   attribute_values.attribute_context_prop_id,
                   attribute_values.key,
                   attribute_values.visibility_change_set_pk,
                   attribute_values.visibility_edit_session_pk,
                   attribute_values.attribute_context_internal_provider_id,
                   attribute_values.attribute_context_external_provider_id,
                   attribute_values.attribute_context_schema_id,
                   attribute_values.attribute_context_schema_variant_id,
                   attribute_values.attribute_context_component_id,
                   attribute_values.attribute_context_system_id,
                   row_to_json(attribute_values.*) AS object
FROM attribute_values
INNER JOIN attribute_value_belongs_to_attribute_value ON
    attribute_value_belongs_to_attribute_value.object_id = attribute_values.id
    AND in_tenancy_v1($1, attribute_value_belongs_to_attribute_value.tenancy_universal,
                          attribute_value_belongs_to_attribute_value.tenancy_billing_account_ids,
                          attribute_value_belongs_to_attribute_value.tenancy_organization_ids,
                          attribute_value_belongs_to_attribute_value.tenancy_workspace_ids)
    AND is_visible_v1($2, attribute_value_belongs_to_attribute_value.visibility_change_set_pk,
                          attribute_value_belongs_to_attribute_value.visibility_edit_session_pk,
                          attribute_value_belongs_to_attribute_value.visibility_deleted)
WHERE in_tenancy_v1($1, attribute_values.tenancy_universal,
                        attribute_values.tenancy_billing_account_ids,
                        attribute_values.tenancy_organization_ids,
                        attribute_values.tenancy_workspace_ids)
    AND is_visible_v1($2, attribute_values.visibility_change_set_pk,
                          attribute_values.visibility_edit_session_pk,
                          attribute_values.visibility_deleted)
    AND in_attribute_context_v1($4, attribute_values.attribute_context_prop_id,
                                    attribute_values.attribute_context_internal_provider_id,
                                    attribute_values.attribute_context_external_provider_id,
                                    attribute_values.attribute_context_schema_id,
                                    attribute_values.attribute_context_schema_variant_id,
                                    attribute_values.attribute_context_component_id,
                                    attribute_values.attribute_context_system_id)
    AND attribute_value_belongs_to_attribute_value.belongs_to_id = $3
ORDER BY attribute_context_prop_id,
         key,
         attribute_values.visibility_change_set_pk DESC,
         attribute_values.visibility_edit_session_pk DESC,
         attribute_values.visibility_deleted DESC,
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_schema_id DESC,
         attribute_context_schema_variant_id DESC,
         attribute_context_component_id DESC,
         attribute_context_system_id DESC;
