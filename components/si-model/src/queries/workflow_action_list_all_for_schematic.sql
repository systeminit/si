SELECT obj as object
FROM workflow_runs
WHERE obj -> 'ctx' -> 'entity' ->> 'id' = $1
AND obj -> 'ctx' -> 'system' ->> 'id' = $2
AND obj -> 'ctx' -> 'workspace' ->> 'id' = $3
ORDER BY created_at DESC
LIMIT 1;

