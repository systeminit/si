ALTER TABLE change_set_pointers ADD COLUMN reviewed_by_user_id ident;
ALTER TABLE change_set_pointers ADD COLUMN reviewed_at TIMESTAMPTZ;
ALTER TABLE change_set_pointers ADD COLUMN merge_requested_at TIMESTAMPTZ;