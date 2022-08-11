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
WHERE in_tenancy_v1($1, attribute_prototypes.tenancy_universal, attribute_prototypes.tenancy_billing_account_ids,
                    attribute_prototypes.tenancy_organization_ids,
                    attribute_prototypes.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_prototypes.visibility_change_set_pk, attribute_prototypes.visibility_deleted_at)
  AND in_attribute_context_v1($3, attribute_prototypes.attribute_context_prop_id,
                              attribute_prototypes.attribute_context_internal_provider_id,
                              attribute_prototypes.attribute_context_external_provider_id,
                              attribute_prototypes.attribute_context_schema_id,
                              attribute_prototypes.attribute_context_schema_variant_id,
                              attribute_prototypes.attribute_context_component_id,
                              attribute_prototypes.attribute_context_system_id)
  AND attribute_prototypes.attribute_context_prop_id = $4
ORDER BY attribute_context_prop_id,
         key,
         visibility_change_set_pk DESC,
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_schema_id DESC,
         attribute_context_schema_variant_id DESC,
         attribute_context_component_id DESC,
         attribute_context_system_id DESC;
