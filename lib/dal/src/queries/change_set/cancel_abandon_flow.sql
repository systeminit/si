UPDATE change_sets
SET status = 'Open', updated_at = now()
WHERE pk = $1
RETURNING updated_at