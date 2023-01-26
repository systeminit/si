CREATE TABLE schemas
(
    pk                          ident primary key                 default ident_create_v1(),
    id                          ident                    not null default ident_create_v1(),
    tenancy_workspace_pks       ident[],
    visibility_change_set_pk    ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    ui_menu_name                text,
    ui_menu_category            ltree,
    ui_hidden                   boolean                  NOT NULL DEFAULT false,
    default_schema_variant_id   ident,
    component_kind              text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('schemas');
SELECT many_to_many_table_create_v1('schema_many_to_many_workspace', 'schemas', 'workspaces');
SELECT many_to_many_table_create_v1('schema_many_to_many_in_menu_for_schema', 'schemas', 'schemas');
SELECT many_to_many_table_create_v1('schema_many_to_many_implements', 'schemas', 'schemas');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('schemas', 'model', 'schema', 'Schema'),
       ('schema_many_to_many_workspace', 'many_to_many', 'schema.workspace', 'Schema <> Workspace'),
       ('schema_many_to_many_in_menu_for_schema', 'many_to_many', 'schema.in_menu_for_schema',
        'Schema <> In Menu For Schema'),
       ('schema_many_to_many_implements', 'many_to_many', 'schema.implements', 'Schema <> Implements');

CREATE OR REPLACE FUNCTION schema_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_component_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           schemas%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO schemas (tenancy_workspace_pks,
                         visibility_change_set_pk, visibility_deleted_at, name, component_kind)
    VALUES (this_tenancy_record.tenancy_workspace_pks,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at,
            this_name, this_component_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
