CREATE TABLE components
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
    kind                        text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('components');
SELECT belongs_to_table_create_v1('component_belongs_to_schema', 'components', 'schemas');
SELECT belongs_to_table_create_v1('component_belongs_to_schema_variant', 'components', 'schema_variants');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('components', 'model', 'component', 'Component'),
       ('component_belongs_to_schema', 'belongs_to', 'component.schema', 'Component <> Schema'),
       ('component_belongs_to_schema_variant', 'belongs_to', 'component.schema_variant', 'Component <> SchemaVariant');


CREATE OR REPLACE FUNCTION component_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO components (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                          tenancy_workspace_ids,
                          visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted, kind)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted, this_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
