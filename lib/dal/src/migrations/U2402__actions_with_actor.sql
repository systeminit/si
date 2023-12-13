ALTER TABLE actions
ADD COLUMN creation_user_id ident;

CREATE OR REPLACE FUNCTION action_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_action_prototype_id ident,
    this_component_id ident,
    this_user_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           actions%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO actions (tenancy_workspace_pk,
                         visibility_change_set_pk,
                         action_prototype_id,
                         change_set_pk,
                         component_id, creation_user_id)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_action_prototype_id,
            this_visibility_record.visibility_change_set_pk,
            this_component_id, this_user_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
