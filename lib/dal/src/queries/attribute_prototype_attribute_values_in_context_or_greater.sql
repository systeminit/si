SELECT DISTINCT ON (attribute_values.id) attribute_values.id,
                                         attribute_values.visibility_change_set_pk,
                                         attribute_values.visibility_edit_session_pk,
                                         attribute_values.visibility_deleted_at,
                                         row_to_json(attribute_values.*) AS object

FROM attribute_values
         INNER JOIN attribute_value_belongs_to_attribute_prototype
                    ON attribute_value_belongs_to_attribute_prototype.object_id = attribute_values.id
                        AND is_visible_v1($2,
                                          attribute_value_belongs_to_attribute_prototype.visibility_change_set_pk,
                                          attribute_value_belongs_to_attribute_prototype.visibility_edit_session_pk,
                                          attribute_value_belongs_to_attribute_prototype.visibility_deleted_at)

WHERE in_tenancy_v1($1,
                    attribute_values.tenancy_universal,
                    attribute_values.tenancy_billing_account_ids,
                    attribute_values.tenancy_organization_ids,
                    attribute_values.tenancy_workspace_ids)
  AND is_visible_v1($2,
                    attribute_values.visibility_change_set_pk,
                    attribute_values.visibility_edit_session_pk,
                    attribute_values.visibility_deleted_at)
  AND attribute_value_belongs_to_attribute_prototype.belongs_to_id = $3
  AND exact_or_more_attribute_read_context_v1($4, attribute_values.attribute_context_prop_id,
                                              attribute_values.attribute_context_internal_provider_id,
                                              attribute_values.attribute_context_external_provider_id,
                                              attribute_values.attribute_context_schema_id,
                                              attribute_values.attribute_context_schema_variant_id,
                                              attribute_values.attribute_context_component_id,
                                              attribute_values.attribute_context_system_id)

ORDER BY attribute_values.id,
         attribute_values.visibility_change_set_pk DESC,
         attribute_values.visibility_edit_session_pk DESC,
         attribute_values.visibility_deleted_at DESC NULLS FIRST;