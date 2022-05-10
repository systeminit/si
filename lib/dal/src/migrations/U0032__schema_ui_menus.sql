CREATE TABLE schema_ui_menus
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1,
    visibility_edit_session_pk  bigint                   NOT NULL DEFAULT -1,
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    name                        text,
    category                    ltree,
    schematic_kind              text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('schema_ui_menus');
SELECT belongs_to_table_create_v1('schema_ui_menu_belongs_to_schema', 'schema_ui_menus', 'schemas');
SELECT many_to_many_table_create_v1('schema_ui_menu_root_schematic_many_to_many_schematic', 'schema_ui_menus',
                                    'schemas');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('schema_ui_menus', 'model', 'schema.ui_menu', 'Schema UI Menu'),
       ('schema_ui_menu_belongs_to_schema', 'belongs_to', 'schema.ui_menu',
        'Schema <> Schema UI Menu'),
       ('schema_ui_menu_root_schematic_many_to_many_schematic', 'many_to_many', 'schema.ui_menu.root_schematic',
        'Schema UI Menu <> Root Schematic');

CREATE OR REPLACE FUNCTION schema_ui_menu_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_schematic_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           schema_ui_menus%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO schema_ui_menus (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                                 tenancy_workspace_ids,
                                 visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted_at,
                                 schematic_kind)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted_at, this_schematic_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
