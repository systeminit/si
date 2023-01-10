UPDATE status_updates
SET finished_at = now(), updated_at = now()
WHERE pk = $1
RETURNING finished_at, updated_at
