SELECT row_to_json(workflow_prototypes.*) AS object
FROM workflow_prototypes_v1($1, $2) AS workflow_prototypes
WHERE workflow_prototypes.func_id = $3
ORDER BY func_id DESC;
