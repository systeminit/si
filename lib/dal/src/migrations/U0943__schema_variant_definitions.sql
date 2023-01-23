CREATE TABLE schema_variant_definitions 
(
    pk                          ident                    PRIMARY KEY DEFAULT ident_create_v1(),
    id                          ident                    NOT NULL DEFAULT ident_create_v1(),
    tenancy_workspace_pk        ident,
    visibility_change_set_pk    ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    category                    text                     NOT NULL,
    menu_name                   text,
    link                        text,
    color                       varchar(6)               NOT NULL DEFAULT '000000',
    component_kind              text                     NOT NULL,
    definition                  text                     NOT NULL DEFAULT ''
);
SELECT standard_model_table_constraints_v1('schema_variant_definitions');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('schema_variant_definitions', 'model', 'schema_variant_definition', 'Schema Variant Definition');

CREATE OR REPLACE FUNCTION schema_variant_definition_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_menu_name text,
    this_category text,
    this_link text,
    this_color text,
    this_component_kind text,
    this_definition text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           schema_variant_definitions%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO schema_variant_definitions (
        tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at,
        name, menu_name, category, link, definition, color, component_kind
    ) VALUES (
        this_tenancy_record.tenancy_workspace_pk,
        this_visibility_record.visibility_change_set_pk,
        this_visibility_record.visibility_deleted_at, this_name,
        this_menu_name, this_category, this_link, this_definition, this_color,
        this_component_kind
    )
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
