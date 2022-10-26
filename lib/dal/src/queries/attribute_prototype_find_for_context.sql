SELECT DISTINCT
    ON (attribute_prototypes.id) attribute_prototypes.id,
                                 attribute_prototypes.attribute_context_prop_id,
                                 attribute_prototypes.visibility_change_set_pk,
                                 attribute_prototypes.visibility_deleted_at,
                                 attribute_prototypes.attribute_context_internal_provider_id,
                                 attribute_prototypes.attribute_context_external_provider_id,
                                 attribute_prototypes.attribute_context_schema_id,
                                 attribute_prototypes.attribute_context_schema_variant_id,
                                 attribute_prototypes.attribute_context_component_id,
                                 attribute_prototypes.attribute_context_system_id,
                                 row_to_json(attribute_prototypes.*) AS object
FROM attribute_prototypes_v1($1, $2) AS attribute_prototypes
WHERE attribute_prototypes.attribute_context_prop_id = $3
  AND attribute_prototypes.attribute_context_internal_provider_id = $4
  AND attribute_prototypes.attribute_context_external_provider_id = $5
  AND attribute_prototypes.attribute_context_schema_id = $6
  AND attribute_prototypes.attribute_context_schema_variant_id = $7
  AND attribute_prototypes.attribute_context_component_id = $8
  AND attribute_prototypes.attribute_context_system_id = $9
ORDER BY attribute_prototypes.id,
         visibility_change_set_pk DESC,
         visibility_deleted_at DESC NULLS FIRST,
         tenancy_universal,
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_schema_id DESC,
         attribute_context_schema_variant_id DESC,
         attribute_context_component_id DESC,
         attribute_context_system_id DESC;
