SELECT DISTINCT ON (id)
    row_to_json(cgp.*) AS object
FROM code_generation_prototypes_v1($1, $2) AS cgp
WHERE
    (
        schema_id = $6
        OR schema_variant_id = $5
        OR component_id = $3
    )
    AND (system_id = $4 OR system_id = -1)
ORDER BY
    id,
    component_id DESC,
    func_id DESC,
    system_id DESC,
    schema_variant_id DESC,
    schema_id DESC;
