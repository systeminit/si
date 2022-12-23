UPDATE component_statuses
SET update_timestamp = now(), update_user_id = $2, updated_at = now()
WHERE pk = $1
RETURNING update_timestamp, updated_at
