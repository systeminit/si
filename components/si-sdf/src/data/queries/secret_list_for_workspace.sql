SELECT obj as object
FROM secrets
WHERE workspace_id = si_id_to_primary_key_v1($1);