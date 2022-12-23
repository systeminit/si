UPDATE status_updates
SET data = $2, updated_at = now()
WHERE pk = $1
RETURNING updated_at
