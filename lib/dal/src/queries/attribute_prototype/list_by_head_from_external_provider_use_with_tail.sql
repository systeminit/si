SELECT DISTINCT ON (
    ap.id,
    apa.head_component_id
)
    row_to_json(attribute_prototypes.*)             AS object
FROM attribute_prototypes_v1($1, $2) AS ap
INNER JOIN attribute_prototype_arguments_v1($1, $2) AS apa
    ON apa.attribute_prototype_id = ap.id
WHERE
    apa.external_provider_id = $3
    AND apa.tail_component_id = $4
ORDER BY
    ap.id,
    apa.head_component_id DESC;
