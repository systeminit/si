SELECT row_to_json(av.*) AS object
FROM attribute_values_v1($1, $2) AS av
INNER JOIN attribute_value_belongs_to_attribute_prototype_v1($1, $2) AS avbtap
    ON avbtap.object_id = av.id
WHERE
    avbtap.belongs_to_id = $3
    AND exact_or_more_attribute_read_context_v1(
        $4, attribute_values.attribute_context_prop_id,
        attribute_values.attribute_context_internal_provider_id,
        attribute_values.attribute_context_external_provider_id,
        attribute_values.attribute_context_schema_id,
        attribute_values.attribute_context_schema_variant_id,
        attribute_values.attribute_context_component_id,
        attribute_values.attribute_context_system_id
    );
