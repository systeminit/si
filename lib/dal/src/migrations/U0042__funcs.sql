CREATE TABLE funcs
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
    backend_kind                text                     NOT NULL,
    backend_response_type       text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('funcs');

CREATE TABLE func_bindings
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
    args                        jsonb                    NOT NULL,
    backend_kind                text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('func_bindings');
SELECT belongs_to_table_create_v1('func_binding_belongs_to_func', 'func_bindings', 'funcs');

CREATE TABLE func_binding_return_values
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
    unprocessed_value           jsonb,
    value                       jsonb
);
SELECT standard_model_table_constraints_v1('func_binding_return_values');
SELECT belongs_to_table_create_v1('func_binding_return_value_belongs_to_func', 'func_binding_return_values', 'funcs');
SELECT belongs_to_table_create_v1('func_binding_return_value_belongs_to_func_binding', 'func_binding_return_values', 'func_bindings');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('funcs', 'model', 'func', 'Func'),
       ('func_bindings', 'model', 'func_binding', 'Func Binding'),
       ('func_binding_belongs_to_func', 'belongs_to', 'func_binding.func', 'Func Binding <> Func'),
       ('func_binding_return_values', 'model', 'func_binding_return_value', 'Func Binding Return Value'),
       ('func_binding_return_value_belongs_to_func', 'belongs_to', 'func_binding_return_value.func', 'Func Binding Return Value <> Func'),
       ('func_binding_return_value_belongs_to_func_binding', 'belongs_to', 'func_binding_return_value.func_binding', 'Func Binding Return Value <> Func Binding')
;

CREATE OR REPLACE FUNCTION func_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_backend_kind text,
    this_backend_response_type text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           funcs%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO funcs (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                       tenancy_workspace_ids,
                       visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted,
                       name, backend_kind, backend_response_type)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted, this_name, this_backend_kind, this_backend_response_type)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION func_binding_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_args jsonb,
    this_backend_kind text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           func_bindings%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO func_bindings (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                               tenancy_workspace_ids,
                               visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted,
                               args, backend_kind)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted, this_args, this_backend_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION func_binding_return_value_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_unprocessed_value jsonb,
    this_value jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           func_binding_return_values%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO func_binding_return_values (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                               tenancy_workspace_ids,
                               visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted,
                               unprocessed_value, value)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted, this_unprocessed_value, this_value)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

