UPDATE change_sets
SET status = $2, updated_at = now()
WHERE pk = $1
RETURNING updated_at