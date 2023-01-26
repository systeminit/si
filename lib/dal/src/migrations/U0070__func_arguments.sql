CREATE TABLE func_arguments
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    name                        text                     NOT NULL,
    kind                        text                     NOT NULL,
    element_kind                text,
    shape                       jsonb,
    func_id                     ident,
    tenancy_billing_account_pks ident[],
    tenancy_organization_pks    ident[],
    tenancy_workspace_pks       ident[],
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP()
);

CREATE UNIQUE INDEX func_argument_name
    ON func_arguments (func_id,
                       name,
                       tenancy_billing_account_pks,
                       tenancy_organization_pks,
                       tenancy_workspace_pks,
                       visibility_change_set_pk,
                       (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;

SELECT standard_model_table_constraints_v1('func_arguments');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('func_arguments', 'model', 'func_argument', 'Func Argument');

CREATE OR REPLACE FUNCTION func_argument_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id ident,
    this_name text,
    this_kind text,
    this_element_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           func_arguments%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO func_arguments (tenancy_billing_account_pks, tenancy_organization_pks,
                                tenancy_workspace_pks, visibility_change_set_pk, visibility_deleted_at, func_id, name,
                                kind, element_kind)
    VALUES (this_tenancy_record.tenancy_billing_account_pks,
            this_tenancy_record.tenancy_organization_pks, this_tenancy_record.tenancy_workspace_pks,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at, this_func_id,
            this_name, this_kind, this_element_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END
$$ LANGUAGE PLPGSQL VOLATILE;
