CREATE TYPE visibility_record_v1 as
(
    visibility_change_set_pk ident,
    visibility_deleted_at    timestamp with time zone
);

CREATE OR REPLACE FUNCTION visibility_json_to_columns_v1(this_visibility jsonb,
                                                         OUT result visibility_record_v1
)
AS
$$
BEGIN
    SELECT *
    FROM jsonb_to_record(this_visibility) AS x(
                                               visibility_change_set_pk ident,
                                               visibility_deleted_at timestamp with time zone
        )
    INTO result;
END ;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION is_visible_v1(
    check_visibility jsonb,
    this_visibility_change_set_pk ident,
    this_visibility_deleted_at timestamp with time zone
)
RETURNS bool
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
CALLED ON NULL INPUT
AS $$
SELECT
    CASE
        WHEN check_visibility -> 'visibility_deleted_at' IS NULL
            OR check_visibility -> 'visibility_deleted_at' = 'null'::jsonb
        THEN this_visibility_deleted_at IS NULL
        ELSE TRUE
    END
    AND (
        this_visibility_change_set_pk = ident_nil_v1()
        OR this_visibility_change_set_pk = (check_visibility ->> 'visibility_change_set_pk')::ident
    )
$$;
