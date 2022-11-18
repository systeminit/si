SELECT DISTINCT ON (
    component_id,
    schema_variant_id,
    schema_id
)
    row_to_json(ap.*) AS object
FROM action_prototypes_v1($1, $2) AS ap
WHERE
    name = $3
    AND schema_id = $5
    AND schema_variant_id = $4
ORDER BY
    component_id DESC,
    schema_variant_id DESC,
    schema_id DESC;
