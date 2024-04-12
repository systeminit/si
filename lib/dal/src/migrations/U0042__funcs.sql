CREATE TABLE func_bindings
(
    pk                       ident primary key                 default ident_create_v1(),
    id                       ident                    not null default ident_create_v1(),
    tenancy_workspace_pk     ident,
    visibility_change_set_pk ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at    timestamp with time zone,
    created_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    args                     json                     NOT NULL,
    backend_kind             text                     NOT NULL,
    code_sha256              text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('func_bindings');
SELECT belongs_to_table_create_v1('func_binding_belongs_to_func', 'func_bindings', 'funcs');

CREATE TABLE func_binding_return_values
(
    pk                       ident primary key                 default ident_create_v1(),
    id                       ident                    not null default ident_create_v1(),
    tenancy_workspace_pk     ident,
    visibility_change_set_pk ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at    timestamp with time zone,
    created_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    unprocessed_value        jsonb,
    value                    jsonb,
    func_id                  ident,
    func_binding_id          ident,
    func_execution_pk        ident
);
CREATE UNIQUE INDEX unique_value_func_binding_return_value_live ON func_binding_return_values (
                                                                                               func_binding_id,
                                                                                               tenancy_workspace_pk,
                                                                                               visibility_change_set_pk);
SELECT standard_model_table_constraints_v1('func_binding_return_values');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('func_bindings', 'model', 'func_binding', 'Func Binding'),
       ('func_binding_belongs_to_func', 'belongs_to', 'func_binding.func', 'Func Binding <> Func'),
       ('func_binding_return_values', 'model', 'func_binding_return_value', 'Func Binding Return Value')
;

CREATE OR REPLACE FUNCTION func_binding_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_args json,
    this_func_id ident,
    this_backend_kind text,
    this_code_sha256 text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           func_bindings%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO func_bindings (tenancy_workspace_pk,
                               visibility_change_set_pk,
                               args,
                               backend_kind,
                               code_sha256)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_args,
            this_backend_kind,
            COALESCE(this_code_sha256, '0'))
    RETURNING * INTO this_new_row;
    PERFORM set_belongs_to_v1(
            'func_binding_belongs_to_func',
            this_tenancy,
            this_visibility,
            this_new_row.id,
            this_func_id
            );

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION func_binding_return_value_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_unprocessed_value jsonb,
    this_value jsonb,
    this_func_id ident,
    this_func_binding_id ident,
    this_func_execution_pk ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           func_binding_return_values%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO func_binding_return_values (tenancy_workspace_pk,
                                            visibility_change_set_pk,
                                            unprocessed_value, value, func_id, func_binding_id, func_execution_pk)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_unprocessed_value, this_value, this_func_id, this_func_binding_id, this_func_execution_pk)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
