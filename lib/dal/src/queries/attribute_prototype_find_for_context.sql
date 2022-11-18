SELECT DISTINCT ON (
    attribute_prototypes.attribute_context_prop_id,
    attribute_prototypes.attribute_context_internal_provider_id,
    attribute_prototypes.attribute_context_external_provider_id
)
    row_to_json(attribute_prototypes.*) AS object
FROM attribute_prototypes_v1($1, $2) AS attribute_prototypes
WHERE
    attribute_prototypes.attribute_context_prop_id = $3
    AND attribute_prototypes.attribute_context_internal_provider_id = $4
    AND attribute_prototypes.attribute_context_external_provider_id = $5
    AND attribute_prototypes.attribute_context_schema_id = $6
    AND attribute_prototypes.attribute_context_schema_variant_id = $7
    AND attribute_prototypes.attribute_context_component_id = $8
ORDER BY
    attribute_context_prop_id DESC,
    attribute_context_internal_provider_id DESC,
    attribute_context_external_provider_id DESC,
    attribute_context_schema_id DESC,
    attribute_context_schema_variant_id DESC,
    attribute_context_component_id DESC,
    attribute_context_system_id DESC;
