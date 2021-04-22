SELECT obj AS object
FROM workflow_run_step_entities
WHERE workflow_run_step_id = si_id_to_primary_key_v1($1);