SELECT DISTINCT ON (attribute_values.attribute_context_prop_id) attribute_values.id,
                              attribute_values.attribute_context_prop_id,
                              attribute_values.visibility_change_set_pk,
                              attribute_values.visibility_edit_session_pk,
                              attribute_values.attribute_context_schema_id,
                              attribute_values.attribute_context_schema_variant_id,
                              attribute_values.attribute_context_component_id,
                              attribute_values.attribute_context_system_id,
                              row_to_json(attribute_values.*) AS object
FROM attribute_values
WHERE in_tenancy_v1($1, attribute_values.tenancy_universal, attribute_values.tenancy_billing_account_ids, attribute_values.tenancy_organization_ids,
                    attribute_values.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_values.visibility_change_set_pk, attribute_values.visibility_edit_session_pk, attribute_values.visibility_deleted)
  AND exact_attribute_context_v1($3, attribute_values.attribute_context_prop_id, attribute_values.attribute_context_schema_id,
                                 attribute_values.attribute_context_schema_variant_id, attribute_values.attribute_context_component_id,
                                 attribute_values.attribute_context_system_id)
ORDER BY attribute_context_prop_id,
      visibility_change_set_pk DESC,
      visibility_edit_session_pk DESC,
      attribute_context_schema_id DESC,
      attribute_context_schema_variant_id DESC,
      attribute_context_component_id DESC,
      attribute_context_system_id DESC;
