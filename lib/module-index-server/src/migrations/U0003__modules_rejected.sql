ALTER TABLE modules
    ADD rejected_at timestamp with time zone,
    ADD rejected_by_display_name text;

