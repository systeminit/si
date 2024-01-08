ALTER TABLE change_sets ADD COLUMN merge_requested_at TIMESTAMPTZ;
ALTER TABLE change_sets ADD COLUMN merge_requested_by_user_id ident;
ALTER TABLE change_sets ADD COLUMN abandon_requested_at TIMESTAMPTZ;
ALTER TABLE change_sets ADD COLUMN abandon_requested_by_user_id ident;
