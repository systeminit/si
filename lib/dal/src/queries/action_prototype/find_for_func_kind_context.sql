SELECT row_to_json(action_prototypes.*) AS object
FROM action_prototypes_v1($1, $2) AS action_prototypes 
WHERE
    action_prototypes.func_id = $3
    AND action_prototypes.kind = $4
    AND action_prototypes.schema_variant_id = $5
ORDER BY
    func_id DESC;
