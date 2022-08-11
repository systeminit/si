CREATE TYPE visibility_record_v1 as
(
    visibility_change_set_pk bigint,
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
                                               visibility_change_set_pk bigint,
                                               visibility_deleted_at timestamp with time zone
        )
    INTO result;
END ;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION is_visible_v1(check_visibility jsonb,
                                         this_visibility_change_set_pk bigint,
                                         this_visibility_deleted_at timestamp with time zone,
                                         OUT result bool
)
AS
$$
DECLARE
    check_visibility_record visibility_record_v1;
    check_deleted_at        bool;
    check_head              bool;
    check_change_set        bool;
BEGIN
    check_visibility_record := visibility_json_to_columns_v1(check_visibility);

    check_deleted_at := CASE
                            WHEN check_visibility_record.visibility_deleted_at IS NULL THEN
                                this_visibility_deleted_at IS NULL
                            ELSE
                                TRUE
        END;

    check_head := (this_visibility_change_set_pk = -1);

    check_change_set := (this_visibility_change_set_pk = check_visibility_record.visibility_change_set_pk);

    result := check_deleted_at AND (
      check_head
      OR
      check_change_set
    );
END ;
$$ LANGUAGE PLPGSQL IMMUTABLE;
