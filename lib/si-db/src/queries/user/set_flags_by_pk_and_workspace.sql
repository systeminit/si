UPDATE user_belongs_to_workspaces
SET flag_map = jsonb_set(
        COALESCE(flag_map, '{}'::jsonb),
        $3,
        $4::jsonb)
WHERE user_pk = $1
  AND workspace_pk = $2
RETURNING flag_map AS object;