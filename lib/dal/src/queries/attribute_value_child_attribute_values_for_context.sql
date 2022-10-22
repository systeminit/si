SELECT DISTINCT ON (av.attribute_context_prop_id, av.key) av.id,
                                                                                      av.attribute_context_prop_id,
                                                                                      av.key,
                                                                                      av.visibility_change_set_pk,

                                                                                      av.attribute_context_internal_provider_id,
                                                                                      av.attribute_context_external_provider_id,
                                                                                      av.attribute_context_schema_id,
                                                                                      av.attribute_context_schema_variant_id,
                                                                                      av.attribute_context_component_id,
                                                                                      av.attribute_context_system_id,
                                                                                      row_to_json(av.*) AS object
FROM attribute_values_v1($1, $2) AS av
         INNER JOIN attribute_value_belongs_to_attribute_value ON
            attribute_value_belongs_to_attribute_value.object_id = av.id
        AND in_tenancy_v1($1, attribute_value_belongs_to_attribute_value.tenancy_universal,
                          attribute_value_belongs_to_attribute_value.tenancy_billing_account_ids,
                          attribute_value_belongs_to_attribute_value.tenancy_organization_ids,
                          attribute_value_belongs_to_attribute_value.tenancy_workspace_ids)
        AND is_visible_v1($2, attribute_value_belongs_to_attribute_value.visibility_change_set_pk,
                          attribute_value_belongs_to_attribute_value.visibility_deleted_at)
WHERE in_attribute_context_v1($4, av)
  AND attribute_value_belongs_to_attribute_value.belongs_to_id = $3
ORDER BY attribute_context_prop_id,
         key,
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_schema_id DESC,
         attribute_context_schema_variant_id DESC,
         attribute_context_component_id DESC,
         attribute_context_system_id DESC;
