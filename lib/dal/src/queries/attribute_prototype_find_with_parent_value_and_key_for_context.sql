SELECT DISTINCT ON (attribute_prototypes.attribute_context_prop_id, attribute_prototypes.key) attribute_prototypes.id,
                                                                                              attribute_prototypes.attribute_context_prop_id,
                                                                                              attribute_prototypes.key,
                                                                                              attribute_prototypes.visibility_change_set_pk,
                                                                                              attribute_prototypes.attribute_context_internal_provider_id,
                                                                                              attribute_prototypes.attribute_context_external_provider_id,
                                                                                              attribute_prototypes.attribute_context_schema_id,
                                                                                              attribute_prototypes.attribute_context_schema_variant_id,
                                                                                              attribute_prototypes.attribute_context_component_id,
                                                                                              attribute_prototypes.attribute_context_system_id,
                                                                                              row_to_json(attribute_prototypes.*) AS object
FROM attribute_prototypes
         INNER JOIN attribute_value_belongs_to_attribute_prototype ON
        attribute_value_belongs_to_attribute_prototype.belongs_to_id = attribute_prototypes.id
         INNER JOIN attribute_values ON
            attribute_values.id = attribute_value_belongs_to_attribute_prototype.object_id
        AND in_tenancy_v1($1, attribute_values.tenancy_universal, attribute_values.tenancy_billing_account_ids,
                          attribute_values.tenancy_organization_ids,
                          attribute_values.tenancy_workspace_ids)
        AND is_visible_v1($2, attribute_values.visibility_change_set_pk, attribute_values.visibility_deleted_at)
         LEFT JOIN attribute_value_belongs_to_attribute_value ON
        attribute_value_belongs_to_attribute_value.object_id = attribute_values.id
         LEFT JOIN attribute_values AS parent_attribute_values ON
            parent_attribute_values.id = attribute_value_belongs_to_attribute_value.belongs_to_id
        AND in_tenancy_v1($1, parent_attribute_values.tenancy_universal,
                          parent_attribute_values.tenancy_billing_account_ids,
                          parent_attribute_values.tenancy_organization_ids,
                          parent_attribute_values.tenancy_workspace_ids)
        AND is_visible_v1($2, parent_attribute_values.visibility_change_set_pk,
                          parent_attribute_values.visibility_deleted_at)
WHERE in_tenancy_v1($1, attribute_prototypes.tenancy_universal, attribute_prototypes.tenancy_billing_account_ids,
                    attribute_prototypes.tenancy_organization_ids,
                    attribute_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_prototypes.visibility_change_set_pk, attribute_prototypes.visibility_deleted_at)
  AND exact_attribute_context_v1($3, attribute_prototypes.attribute_context_prop_id,
                                 attribute_prototypes.attribute_context_internal_provider_id,
                                 attribute_prototypes.attribute_context_external_provider_id,
                                 attribute_prototypes.attribute_context_schema_id,
                                 attribute_prototypes.attribute_context_schema_variant_id,
                                 attribute_prototypes.attribute_context_component_id,
                                 attribute_prototypes.attribute_context_system_id)
  AND CASE
          WHEN $4::bigint IS NULL THEN parent_attribute_values.id IS NULL
          ELSE parent_attribute_values.id = $4::bigint
    END
  AND CASE
          WHEN $5::text IS NULL THEN attribute_prototypes.key IS NULL
          ELSE attribute_prototypes.key = $5::text
    END
ORDER BY attribute_context_prop_id,
         key,
         visibility_change_set_pk DESC,
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_schema_id DESC,
         attribute_context_schema_variant_id DESC,
         attribute_context_component_id DESC,
         attribute_context_system_id DESC;
