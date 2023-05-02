CREATE TABLE command_prototypes
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_workspace_pk        ident,
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    func_id                     ident                   NOT NULL,
    component_id                ident                   NOT NULL,
    schema_variant_id           ident                   NOT NULL
);

CREATE UNIQUE INDEX unique_commmand_prototypes_for_schema_variants_and_components
    ON command_prototypes (func_id,
                            component_id,
                            schema_variant_id,
                            tenancy_workspace_pk,
                            visibility_change_set_pk);
SELECT standard_model_table_constraints_v1('command_prototypes');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('command_prototypes', 'model', 'command_prototype', 'Command Prototype');

CREATE OR REPLACE FUNCTION command_prototype_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id ident,
    this_component_id ident,
    this_schema_variant_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           command_prototypes%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO command_prototypes (tenancy_workspace_pk,
                                     visibility_change_set_pk,
                                     func_id,
                                     component_id,
                                     schema_variant_id)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_func_id,
            this_component_id,
            this_schema_variant_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
