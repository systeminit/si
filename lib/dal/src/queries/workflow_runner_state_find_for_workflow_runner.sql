SELECT row_to_json(workflow_runner_states.*) AS object
FROM workflow_runner_states_v1($1, $2) AS workflow_runner_states
WHERE workflow_runner_id = $3
