SELECT DISTINCT ON (id)
    row_to_json(cgr.*) as object
FROM code_generation_resolvers_v1($1, $2) AS cgr
WHERE
    code_generation_prototype_id = $3
    AND component_id = $4
ORDER BY
    id,
    component_id DESC,
    system_id DESC,
    schema_variant_id DESC,
    schema_id DESC;
