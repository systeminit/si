SELECT DISTINCT ON (qualification_prototypes.func_id)
    row_to_json(qualification_prototypes.*) AS object
FROM qualification_prototypes_v1($1, $2) AS qualification_prototypes
WHERE
    qualification_prototypes.schema_id = $5
    OR qualification_prototypes.schema_variant_id = $4
    OR qualification_prototypes.component_id = $3
ORDER BY qualification_prototypes.func_id,
         component_id DESC,
         func_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
