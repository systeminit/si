ALTER TABLE fix_batches
ADD COLUMN actors TEXT;

CREATE OR REPLACE FUNCTION fix_batch_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_author text,
    this_actors text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           fix_batches%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

INSERT INTO fix_batches (tenancy_workspace_pk, visibility_change_set_pk, author, actors)
VALUES (this_tenancy_record.tenancy_workspace_pk,
        this_visibility_record.visibility_change_set_pk, this_author, this_actors)
    RETURNING * INTO this_new_row;

object := row_to_json(this_new_row);
END
$$ LANGUAGE PLPGSQL VOLATILE;
