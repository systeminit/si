SELECT row_to_json(workflow_prototypes.*) AS object
FROM workflow_prototypes_v1($1, $2) AS workflow_prototypes
WHERE
    workflow_prototypes.schema_id = $5
    AND workflow_prototypes.schema_variant_id = $4
    AND workflow_prototypes.component_id = $3
ORDER BY
    component_id DESC,
    func_id DESC,
    schema_variant_id DESC,
    schema_id DESC;
