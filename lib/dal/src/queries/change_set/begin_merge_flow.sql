UPDATE change_sets
SET status = 'NeedsApproval', updated_at = now(), merge_requested_at = now(), merge_requested_by_user_id = $2
WHERE pk = $1
RETURNING updated_at