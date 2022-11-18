SELECT row_to_json(workflow_runners.*) as object
FROM workflow_runners_v1($1, $2) as workflow_runners
WHERE
    workflow_runners.workflow_prototype_id = $3
    AND (
        workflow_runners.component_id = $4
        OR workflow_runners.schema_id = $5
        OR workflow_runners.schema_variant_id = $6
    )
ORDER BY
    component_id DESC,
    schema_variant_id DESC,
    schema_id DESC;
