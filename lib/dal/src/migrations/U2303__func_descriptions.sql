DELETE FROM standard_models WHERE table_name = 'func_descriptions';
DROP TABLE func_descriptions CASCADE;

CREATE OR REPLACE FUNCTION fix_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
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
                       component_id, action_prototype_id, action_kind)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_component_id, this_action_prototype_id, this_action_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION fix_resolver_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_action_prototype_id ident,
    this_success bool,
    this_last_fix_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           fix_resolvers%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO fix_resolvers (tenancy_workspace_pk,
                               visibility_change_set_pk,
                               action_prototype_id,
                               success,
                               last_fix_id)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_action_prototype_id,
            this_success,
            this_last_fix_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

ALTER TABLE fixes DROP COLUMN attribute_value_id;
ALTER TABLE fix_resolvers DROP COLUMN attribute_value_id;
