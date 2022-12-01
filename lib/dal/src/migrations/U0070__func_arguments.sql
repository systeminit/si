CREATE TABLE func_arguments
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    name                        text                     NOT NULL,
    kind                        text                     NOT NULL,
    element_kind                text,
    shape                       jsonb,
    func_id                     bigint,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1,
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX func_argument_name
    ON func_arguments (func_id,
                       name,
                       tenancy_universal,
                       tenancy_billing_account_ids,
                       tenancy_organization_ids,
                       tenancy_workspace_ids,
                       visibility_change_set_pk,
                       (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;

SELECT standard_model_table_constraints_v1('func_arguments');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('func_arguments', 'model', 'func_argument', 'Func Argument');

CREATE OR REPLACE FUNCTION func_argument_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id bigint,
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

    INSERT INTO func_arguments (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                                tenancy_workspace_ids, visibility_change_set_pk, visibility_deleted_at, func_id, name,
                                kind, element_kind)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at, this_func_id,
            this_name, this_kind, this_element_kind)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END
$$ LANGUAGE PLPGSQL VOLATILE;
