SELECT DISTINCT ON (confirmation_prototypes.func_id)
    row_to_json(confirmation_prototypes.*) AS object
FROM confirmation_prototypes_v1($1, $2) AS confirmation_prototypes
WHERE
    confirmation_prototypes.schema_id = $5
    OR confirmation_prototypes.schema_variant_id = $4
    OR confirmation_prototypes.component_id = $3
ORDER BY
    confirmation_prototypes.func_id,
    component_id DESC,
    func_id DESC,
    schema_variant_id DESC,
    schema_id DESC;
