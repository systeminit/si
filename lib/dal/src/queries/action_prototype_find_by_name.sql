SELECT DISTINCT ON (
    component_id,
    system_id,
    schema_variant_id,
    schema_id
)
    row_to_json(ap.*) AS object
FROM action_prototypes_v1($1, $2) AS ap
WHERE
    name = $3
    AND schema_id = $6
    AND schema_variant_id = $5
    AND (system_id = $4 OR system_id = -1)
ORDER BY
    component_id DESC,
    system_id DESC,
    schema_variant_id DESC,
    schema_id DESC;
