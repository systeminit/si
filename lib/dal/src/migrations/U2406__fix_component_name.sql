ALTER TABLE fixes ADD COLUMN component_name TEXT;
UPDATE fixes SET component_name = '';
ALTER TABLE fixes ALTER COLUMN component_name SET NOT NULL;

CREATE OR REPLACE FUNCTION fix_create_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_component_name text,
    this_action_prototype_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           fixes%ROWTYPE;
    this_action_kind       text;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    SELECT
    INTO STRICT this_action_kind
    kind FROM action_prototypes_v1($1, $2) where id = this_action_prototype_id;

    INSERT INTO fixes (tenancy_workspace_pk, visibility_change_set_pk,
                       component_id, component_name, action_prototype_id, action_kind)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_component_id, this_component_name, this_action_prototype_id, this_action_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END
$$ LANGUAGE PLPGSQL VOLATILE;
