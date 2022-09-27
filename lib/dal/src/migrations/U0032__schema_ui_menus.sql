CREATE TABLE schema_ui_menus
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1,
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    name                        text                     NOT NULL,
    category                    text                     NOT NULL,
    diagram_kind                text                     NOT NULL
);

CREATE UNIQUE INDEX unique_schema_ui_menus
    ON schema_ui_menus (name,
                        category,
                        diagram_kind,
                        visibility_change_set_pk,
                        (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;

SELECT standard_model_table_constraints_v1('schema_ui_menus');
SELECT belongs_to_table_create_v1('schema_ui_menu_belongs_to_schema', 'schema_ui_menus', 'schemas');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('schema_ui_menus', 'model', 'schema.ui_menu', 'Schema UI Menu'),
       ('schema_ui_menu_belongs_to_schema', 'belongs_to', 'schema.ui_menu',
        'Schema <> Schema UI Menu');

CREATE OR REPLACE FUNCTION schema_ui_menu_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_category text,
    this_diagram_kind text,
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
                                 visibility_change_set_pk, visibility_deleted_at,
                                 name, category, diagram_kind)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at,
            this_name, this_category, this_diagram_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
