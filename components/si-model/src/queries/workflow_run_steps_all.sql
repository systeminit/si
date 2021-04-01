SELECT obj AS object
FROM workflow_run_steps
WHERE workflow_run_id = si_id_to_primary_key_v1($1);