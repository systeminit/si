ALTER TABLE modules
    ADD is_builtin_at timestamp with time zone,
    ADD is_builtin_at_by_display_name text;