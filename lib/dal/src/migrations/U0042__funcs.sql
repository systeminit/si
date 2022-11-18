CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE funcs
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
    display_name                text,
    description                 text,
    link                        text,
    hidden                      bool                     NOT NULL DEFAULT FALSE,
    backend_kind                text                     NOT NULL,
    backend_response_type       text                     NOT NULL,
    handler                     text,
    code_base64                 text,
    code_sha256                 text GENERATED ALWAYS AS (COALESCE(ENCODE(DIGEST(code_base64, 'sha256'), 'hex'), '0')) STORED,
    CONSTRAINT unique_name unique (name, tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                                   tenancy_workspace_ids, visibility_change_set_pk, visibility_deleted_at)
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
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    args                        json                     NOT NULL,
    backend_kind                text                     NOT NULL,
    code_sha256                 text                     NOT NULL
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
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    unprocessed_value           jsonb,
    value                       jsonb,
    func_execution_pk           bigint
);
SELECT standard_model_table_constraints_v1('func_binding_return_values');
SELECT belongs_to_table_create_v1('func_binding_return_value_belongs_to_func', 'func_binding_return_values', 'funcs');
SELECT belongs_to_table_create_v1('func_binding_return_value_belongs_to_func_binding', 'func_binding_return_values',
                                  'func_bindings');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('funcs', 'model', 'func', 'Func'),
       ('func_bindings', 'model', 'func_binding', 'Func Binding'),
       ('func_binding_belongs_to_func', 'belongs_to', 'func_binding.func', 'Func Binding <> Func'),
       ('func_binding_return_values', 'model', 'func_binding_return_value', 'Func Binding Return Value'),
       ('func_binding_return_value_belongs_to_func', 'belongs_to', 'func_binding_return_value.func',
        'Func Binding Return Value <> Func'),
       ('func_binding_return_value_belongs_to_func_binding', 'belongs_to', 'func_binding_return_value.func_binding',
        'Func Binding Return Value <> Func Binding')
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
                       visibility_change_set_pk, visibility_deleted_at,
                       name, backend_kind, backend_response_type)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at, this_name, this_backend_kind, this_backend_response_type)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION func_binding_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_args json,
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

    INSERT INTO func_bindings (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                               tenancy_workspace_ids,
                               visibility_change_set_pk, visibility_deleted_at,
                               args, backend_kind, code_sha256)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_args, this_backend_kind, COALESCE(this_code_sha256, '0'))
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

-- There is a potential race condition in this function - if two bindings are created at the
-- exact same time, there could wind up being two identical FuncBindings in the database.
-- We will account for this later, when we are reading the data back, by having a consistent
-- ordering. (We order by `id` ASC, and limit 1)
CREATE OR REPLACE FUNCTION func_binding_find_or_create_v1(
    this_read_tenancy jsonb,
    this_write_tenancy jsonb,
    this_visibility jsonb,
    this_args json,
    this_backend_kind text,
    this_func_id bigint,
    OUT object json, OUT created bool) AS
$$
DECLARE
    this_code_sha256           text;

    -- Please no hate, this is a hack to be able to SELECT INTO object while sorting by visibility
    dummy1                     bigint;
    dummy2                     bigint;
    dummy3                     bigint;
BEGIN
    created := false;

    SELECT DISTINCT ON (id) code_sha256
    FROM funcs
    WHERE in_tenancy_and_visible_v1(
            this_read_tenancy,
            this_visibility,
            funcs
        )
      AND id = this_func_id
    ORDER BY id, visibility_change_set_pk DESC, visibility_deleted_at DESC NULLS FIRST
    INTO this_code_sha256;

    SELECT DISTINCT ON (func_bindings.id) func_bindings.id,
                                          func_bindings.visibility_change_set_pk,
                                          func_bindings.visibility_deleted_at,
                                          row_to_json(func_bindings.*)
    FROM func_bindings
             INNER JOIN func_binding_belongs_to_func ON
                func_binding_belongs_to_func.object_id = func_bindings.id
            AND func_binding_belongs_to_func.belongs_to_id = this_func_id
    WHERE func_bindings.args::jsonb = this_args::jsonb
      AND func_bindings.backend_kind = this_backend_kind
      AND func_bindings.code_sha256 = this_code_sha256
      AND in_tenancy_and_visible_v1(this_read_tenancy, this_visibility, func_bindings)
    ORDER BY func_bindings.id,
             func_bindings.visibility_change_set_pk DESC,
             func_bindings.visibility_deleted_at DESC NULLS FIRST
    INTO dummy1, dummy2, dummy3, object;

    IF object IS NULL THEN
        created := true;

        SELECT *
        FROM func_binding_create_v1(this_write_tenancy, this_visibility, this_args, this_backend_kind, this_code_sha256)
        INTO object;

        PERFORM set_belongs_to_v1('func_binding_belongs_to_func',
                                  this_write_tenancy,
                                  this_visibility,
                                  (object ->> 'id')::bigint,
                                  this_func_id);
    END IF;
END;
$$ LANGUAGE PLPGSQL VOLATILE;


CREATE OR REPLACE FUNCTION func_binding_return_value_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_unprocessed_value jsonb,
    this_value jsonb,
    this_func_execution_pk bigint,
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
                                            visibility_change_set_pk, visibility_deleted_at,
                                            unprocessed_value, value, func_execution_pk)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at, this_unprocessed_value, this_value, this_func_execution_pk)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

-- TODO(nick,fletcher): we likely need "func_binding_return_value_create_or_update_v1" at some point.
