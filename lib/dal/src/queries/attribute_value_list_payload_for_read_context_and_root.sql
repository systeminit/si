SELECT DISTINCT ON (
    attribute_values.attribute_context_prop_id,
    attribute_value_belongs_to_attribute_value.belongs_to_id,
    attribute_values.key
    ) attribute_values.id,
      attribute_values.visibility_change_set_pk,
      attribute_values.visibility_edit_session_pk,
      attribute_values.attribute_context_prop_id,
      attribute_values.attribute_context_internal_provider_id,
      attribute_values.attribute_context_external_provider_id,
      attribute_values.attribute_context_schema_id,
      attribute_values.attribute_context_schema_variant_id,
      attribute_values.attribute_context_component_id,
      attribute_values.attribute_context_system_id,
      parent_attribute_values.id              AS parent_attribute_value_id,
      row_to_json(attribute_values.*)         AS attribute_value_object,
      row_to_json(props.*)                    AS prop_object,
      row_to_json(func_binding_return_values) AS object

FROM attribute_values
         INNER JOIN props ON
    props.id = attribute_values.attribute_context_prop_id
         INNER JOIN func_binding_return_values ON
            func_binding_return_values.id = attribute_values.func_binding_return_value_id
        AND is_visible_v1($2, func_binding_return_values.visibility_change_set_pk,
                          func_binding_return_values.visibility_edit_session_pk,
                          func_binding_return_values.visibility_deleted_at)
         LEFT JOIN attribute_value_belongs_to_attribute_value ON
            attribute_values.id = attribute_value_belongs_to_attribute_value.object_id
        AND is_visible_v1($2, attribute_value_belongs_to_attribute_value.visibility_change_set_pk,
                          attribute_value_belongs_to_attribute_value.visibility_edit_session_pk,
                          attribute_value_belongs_to_attribute_value.visibility_deleted_at)
         LEFT JOIN attribute_values AS parent_attribute_values ON
            attribute_value_belongs_to_attribute_value.belongs_to_id = parent_attribute_values.id
        AND is_visible_v1($2, parent_attribute_values.visibility_change_set_pk,
                          parent_attribute_values.visibility_edit_session_pk,
                          parent_attribute_values.visibility_deleted_at)

WHERE in_tenancy_v1($1, attribute_values.tenancy_universal, attribute_values.tenancy_billing_account_ids,
                    attribute_values.tenancy_organization_ids,
                    attribute_values.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_values.visibility_change_set_pk, attribute_values.visibility_edit_session_pk,
                    attribute_values.visibility_deleted_at)
  AND in_attribute_context_v1($3, attribute_values.attribute_context_prop_id,
                              attribute_values.attribute_context_internal_provider_id,
                              attribute_values.attribute_context_external_provider_id,
                              attribute_values.attribute_context_schema_id,
                              attribute_values.attribute_context_schema_variant_id,
                              attribute_values.attribute_context_component_id,
                              attribute_values.attribute_context_system_id)
  AND attribute_values.id IN (
    WITH RECURSIVE recursive_attribute_values AS (
        SELECT $4::bigint AS attribute_value_id
        UNION ALL
        SELECT aba.object_id AS attribute_value_id
        FROM attribute_value_belongs_to_attribute_value AS aba
                 JOIN recursive_attribute_values
                      ON aba.belongs_to_id = recursive_attribute_values.attribute_value_id
                      WHERE in_tenancy_v1($1, aba.tenancy_universal,
                                              aba.tenancy_billing_account_ids,
                                              aba.tenancy_organization_ids,
                                              aba.tenancy_workspace_ids)
                            AND is_visible_v1($2, aba.visibility_change_set_pk,
                                                  aba.visibility_edit_session_pk,
                                                  aba.visibility_deleted_at)
    )
    SELECT attribute_value_id
    FROM recursive_attribute_values
)

ORDER BY attribute_values.attribute_context_prop_id,
         attribute_value_belongs_to_attribute_value.belongs_to_id,
         attribute_values.key,
         visibility_change_set_pk DESC,
         visibility_edit_session_pk DESC,
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_schema_id DESC,
         attribute_context_schema_variant_id DESC,
         attribute_context_component_id DESC,
         attribute_context_system_id DESC;
