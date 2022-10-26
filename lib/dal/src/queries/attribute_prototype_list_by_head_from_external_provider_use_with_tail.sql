SELECT DISTINCT ON (attribute_prototypes.id, attribute_prototype_arguments.head_component_id)
    row_to_json(attribute_prototypes.*)             AS object
FROM attribute_prototypes_v1($1, $2) AS ap
INNER JOIN attribute_prototype_arguments_v1($1, $2) AS apa
    ON apa.attribute_prototype_id = ap.id
WHERE
    attribute_prototype_arguments.external_provider_id = $3
    AND attribute_prototype_arguments.tail_component_id = $4
ORDER BY
    attribute_prototypes.id,
    attribute_prototype_arguments.head_component_id DESC;
