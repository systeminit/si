UPDATE change_sets
SET status = 'Abandoned', updated_at = now()
WHERE pk = $1
RETURNING updated_at