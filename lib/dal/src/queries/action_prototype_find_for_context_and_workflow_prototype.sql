SELECT row_to_json(action_prototypes.*) AS object
FROM action_prototypes_v1($1, $2) AS action_prototypes 
WHERE
    action_prototypes.schema_id = $5
    AND action_prototypes.schema_variant_id = $4
    AND action_prototypes.component_id = $3
    AND action_prototypes.workflow_prototype_id = $6
ORDER BY
    component_id DESC,
    schema_variant_id DESC,
    schema_id DESC,
    workflow_prototype_id DESC;
