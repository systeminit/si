SELECT DISTINCT ON (attribute_values.attribute_context_prop_id)
                   attribute_values.attribute_context_prop_id,
                   attribute_values.visibility_change_set_pk,
                   attribute_values.visibility_edit_session_pk,
                   attribute_values.visibility_deleted,
                   attribute_values.attribute_context_schema_id,
                   attribute_values.attribute_context_schema_variant_id,
                   attribute_values.attribute_context_component_id,
                   attribute_values.attribute_context_system_id,
                   row_to_json(attribute_values.*) AS object
FROM attribute_values

LEFT JOIN attribute_value_belongs_to_attribute_value ON
    attribute_value_belongs_to_attribute_value.object_id = attribute_values.id
    AND in_tenancy_v1($1, attribute_value_belongs_to_attribute_value.tenancy_universal,
        attribute_value_belongs_to_attribute_value.tenancy_billing_account_ids,
        attribute_value_belongs_to_attribute_value.tenancy_organization_ids,
        attribute_value_belongs_to_attribute_value.tenancy_workspace_ids)
    AND is_visible_v1($2, attribute_value_belongs_to_attribute_value.visibility_change_set_pk,
        attribute_value_belongs_to_attribute_value.visibility_edit_session_pk,
        attribute_value_belongs_to_attribute_value.visibility_deleted)

WHERE in_tenancy_v1($1, attribute_values.tenancy_universal, attribute_values.tenancy_billing_account_ids, attribute_values.tenancy_organization_ids,
                        attribute_values.tenancy_workspace_ids)
    AND is_visible_v1($2, attribute_values.visibility_change_set_pk, attribute_values.visibility_edit_session_pk, attribute_values.visibility_deleted)
    AND in_attribute_context_v1($3, attribute_values.attribute_context_prop_id, attribute_values.attribute_context_schema_id, attribute_values.attribute_context_schema_variant_id,
                                    attribute_values.attribute_context_component_id, attribute_values.attribute_context_system_id)
    AND CASE
        WHEN $4::bigint IS NULL THEN attribute_value_belongs_to_attribute_value.belongs_to_id IS NULL
        ELSE attribute_value_belongs_to_attribute_value.belongs_to_id = $4::bigint
    END
    AND CASE
        WHEN $5::text IS NULL THEN attribute_values.key IS NULL
        ELSE attribute_values.key = $5::text
    END

ORDER BY attribute_context_prop_id,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC,
         visibility_deleted DESC,
         attribute_context_schema_id DESC,
         attribute_context_schema_variant_id DESC,
         attribute_context_component_id DESC,
         attribute_context_system_id DESC;
