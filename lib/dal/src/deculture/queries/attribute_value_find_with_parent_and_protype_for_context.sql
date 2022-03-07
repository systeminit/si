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

-- Scope by attribute prototype. We need these for handling elements in arrays and values in maps.
INNER JOIN attribute_value_belongs_to_attribute_prototype ON
  attribute_value_belongs_to_attribute_prototype.object_id = attribute_values.id
  AND in_tenancy_v1($1, attribute_value_belongs_to_attribute_prototype.tenancy_universal,
                    attribute_value_belongs_to_attribute_prototype.tenancy_billing_account_ids,
                    attribute_value_belongs_to_attribute_prototype.tenancy_organization_ids,
                    attribute_value_belongs_to_attribute_prototype.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_value_belongs_to_attribute_prototype.visibility_change_set_pk,
                    attribute_value_belongs_to_attribute_prototype.visibility_edit_session_pk,
                    attribute_value_belongs_to_attribute_prototype.visibility_deleted)
INNER JOIN attribute_prototypes ON
  attribute_prototypes.id = attribute_value_belongs_to_attribute_prototype.belongs_to_id
  AND in_tenancy_v1($1, attribute_prototypes.tenancy_universal,
                    attribute_prototypes.tenancy_billing_account_ids,
                    attribute_prototypes.tenancy_organization_ids,
                    attribute_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_prototypes.visibility_change_set_pk,
                    attribute_prototypes.visibility_edit_session_pk,
                    attribute_prototypes.visibility_deleted)

-- Handle parentage. We need to use LEFT JOINs here to not wipe out attribute values that do not have relevant parents.
LEFT JOIN attribute_value_belongs_to_attribute_value ON
  attribute_value_belongs_to_attribute_value.object_id = attribute_values.id
  AND in_tenancy_v1($1, attribute_value_belongs_to_attribute_value.tenancy_universal,
                    attribute_value_belongs_to_attribute_value.tenancy_billing_account_ids,
                    attribute_value_belongs_to_attribute_value.tenancy_organization_ids,
                    attribute_value_belongs_to_attribute_value.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_value_belongs_to_attribute_value.visibility_change_set_pk,
                    attribute_value_belongs_to_attribute_value.visibility_edit_session_pk,
                    attribute_value_belongs_to_attribute_value.visibility_deleted)
LEFT JOIN attribute_values AS parent_attribute_values ON
  parent_attribute_values.id = attribute_value_belongs_to_attribute_value.belongs_to_id
  AND in_tenancy_v1($1, parent_attribute_values.tenancy_universal,
                    parent_attribute_values.tenancy_billing_account_ids,
                    parent_attribute_values.tenancy_organization_ids,
                    parent_attribute_values.tenancy_workspace_ids)
  AND is_visible_v1($2, parent_attribute_values.visibility_change_set_pk,
                    parent_attribute_values.visibility_edit_session_pk,
                    parent_attribute_values.visibility_deleted)

WHERE in_tenancy_v1($1, attribute_values.tenancy_universal, attribute_values.tenancy_billing_account_ids, attribute_values.tenancy_organization_ids,
                    attribute_values.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_values.visibility_change_set_pk, attribute_values.visibility_edit_session_pk, attribute_values.visibility_deleted)
  AND exact_attribute_context_v1($3, attribute_values.attribute_context_prop_id, attribute_values.attribute_context_schema_id, attribute_values.attribute_context_schema_variant_id,
                                 attribute_values.attribute_context_component_id, attribute_values.attribute_context_system_id)
  AND attribute_prototypes.id = $4
  AND CASE
      WHEN $5::bigint IS NULL THEN parent_attribute_values.id IS NULL
      ELSE parent_attribute_values.id = $5::bigint
  END
ORDER BY attribute_context_prop_id,
      visibility_change_set_pk DESC,
      visibility_edit_session_pk DESC,
      attribute_context_schema_id DESC
      attribute_context_schema_variant_id DESC,
      attribute_context_component_id DESC,
      attribute_context_system_id DESC;
