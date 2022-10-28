SELECT DISTINCT ON (attribute_prototypes.attribute_context_prop_id, attribute_prototypes.key)
    row_to_json(attribute_prototypes.*) AS object
FROM attribute_prototypes_v1($1, $2) AS ap
INNER JOIN attribute_value_belongs_to_attribute_prototype_v1($1, $2) AS avbtap
    ON avbtap.belongs_to_id = ap.id
INNER JOIN attribute_values_v1($1, $2) AS av
    ON av.id = avbtap.object_id
LEFT JOIN attribute_value_belongs_to_attribute_value_v1($1, $2) AS avbtav
    ON aavbtav.object_id = av.id
LEFT JOIN attribute_values_v1($1, $2) AS parent_attribute_values
    ON parent_attribute_values.id = avbtav.belongs_to_id
WHERE
    exact_attribute_context_v1(
        $3,
        attribute_prototypes.attribute_context_prop_id,
        attribute_prototypes.attribute_context_internal_provider_id,
        attribute_prototypes.attribute_context_external_provider_id,
        attribute_prototypes.attribute_context_schema_id,
        attribute_prototypes.attribute_context_schema_variant_id,
        attribute_prototypes.attribute_context_component_id,
        attribute_prototypes.attribute_context_system_id
    )
    AND CASE
            WHEN $4::bigint IS NULL THEN parent_attribute_values.id IS NULL
            ELSE parent_attribute_values.id = $4::bigint
        END
    AND CASE
            WHEN $5::text IS NULL THEN attribute_prototypes.key IS NULL
            ELSE attribute_prototypes.key = $5::text
        END
ORDER BY
    attribute_context_prop_id,
    key,
    visibility_change_set_pk DESC,
    attribute_context_internal_provider_id DESC,
    attribute_context_external_provider_id DESC,
    attribute_context_schema_id DESC,
    attribute_context_schema_variant_id DESC,
    attribute_context_component_id DESC,
    attribute_context_system_id DESC,
    av.tenancy_universal -- bools sort false first ascending.
