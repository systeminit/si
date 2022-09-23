SELECT DISTINCT
ON (func_arguments.id) func_arguments.id,
    func_arguments.visibility_change_set_pk,
    func_arguments.visibility_deleted_at,
    row_to_json(func_arguments.*) AS object
FROM func_arguments
WHERE in_tenancy_and_visible_v1($1, $2, func_arguments)
    AND func_arguments.name = $3
    AND func_arguments.func_id = $4
ORDER BY func_arguments.id,
    visibility_change_set_pk DESC,
    visibility_deleted_at DESC NULLS FIRST
LIMIT 1
