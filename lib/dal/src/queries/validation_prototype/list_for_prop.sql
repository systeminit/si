SELECT row_to_json(validation_prototypes.*) AS object
FROM validation_prototypes_v1($1, $2) as validation_prototypes
WHERE validation_prototypes.prop_id = $3
ORDER BY validation_prototypes.id,
         prop_id DESC,
         func_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
