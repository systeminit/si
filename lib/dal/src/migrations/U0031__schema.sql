CREATE TABLE schemas
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1,
    visibility_edit_session_pk  bigint                   NOT NULL DEFAULT -1,
    visibility_deleted          bool,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    name                        text                     NOT NULL,
    kind                        text                     NOT NULL,
    ui_menu_name                text,
    ui_menu_category            ltree,
    ui_hidden                   boolean                  NOT NULL DEFAULT false,
    default_schema_variant_id   bigint
);
SELECT standard_model_table_constraints_v1('schemas');
SELECT many_to_many_table_create_v1('schema_many_to_many_billing_account', 'schemas', 'billing_accounts');
SELECT many_to_many_table_create_v1('schema_many_to_many_organization', 'schemas', 'organizations');
SELECT many_to_many_table_create_v1('schema_many_to_many_workspace', 'schemas', 'workspaces');
SELECT many_to_many_table_create_v1('schema_many_to_many_in_menu_for_schema', 'schemas', 'schemas');
SELECT many_to_many_table_create_v1('schema_many_to_many_implements', 'schemas', 'schemas');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('schemas', 'model', 'schema', 'Schema'),
       ('schema_many_to_many_billing_account', 'many_to_many', 'schema.billing_account', 'Schema <> Billing Account'),
       ('schema_many_to_many_organization', 'many_to_many', 'schema.organization', 'Schema <> Organization'),
       ('schema_many_to_many_workspace', 'many_to_many', 'schema.workspace', 'Schema <> Workspace'),
       ('schema_many_to_many_in_menu_for_schema', 'many_to_many', 'schema.in_menu_for_schema',
        'Schema <> In Menu For Schema'),
       ('schema_many_to_many_implements', 'many_to_many', 'schema.implements', 'Schema <> Implements');

CREATE OR REPLACE FUNCTION schema_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           schemas%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO schemas (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                         tenancy_workspace_ids,
                         visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted, name, kind)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted, this_name, this_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;