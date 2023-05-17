CREATE TABLE fix_resolvers
(
    pk                       ident primary key                 default ident_create_v1(),
    id                       ident                    not null default ident_create_v1(),
    tenancy_workspace_pk     ident,
    visibility_change_set_pk ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at    timestamp with time zone,
    created_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    action_prototype_id      ident                    NOT NULL,
    attribute_value_id       ident                    NOT NULL,
    success                  bool,
    last_fix_id              ident                    NOT NULL
);

CREATE UNIQUE INDEX unique_fix_resolvers
    ON fix_resolvers (attribute_value_id,
                      tenancy_workspace_pk,
                      visibility_change_set_pk);

SELECT standard_model_table_constraints_v1('fix_resolvers');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('fix_resolvers', 'model', 'fix_resolver', 'Fix Resolver');

CREATE OR REPLACE FUNCTION fix_resolver_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_action_prototype_id ident,
    this_attribute_value_id ident,
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
                               attribute_value_id,
                               success,
                               last_fix_id)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_action_prototype_id,
            this_attribute_value_id,
            this_success,
            this_last_fix_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
