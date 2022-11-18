SELECT row_to_json(cgp.*) AS object
FROM code_generation_prototypes_v1($1, $2) AS cgp
WHERE schema_variant_id = $3
ORDER BY
    prop_id DESC,
    schema_variant_id DESC
