UPDATE change_sets
SET status = 'NeedsAbandonApproval', updated_at = now(),  abandon_requested_at = now(), abandon_requested_by_user_id = $2
WHERE pk = $1
RETURNING updated_at