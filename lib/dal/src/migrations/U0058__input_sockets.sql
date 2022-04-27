CREATE TABLE input_sockets
(
    pk bigserial PRIMARY KEY,
    id bigserial NOT NULL,
    tenancy_universal bool NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids bigint[],
    tenancy_workspace_ids bigint[],
    visibility_change_set_pk bigint,
    visibility_edit_session_pk bigint,
    visibility_deleted bool,
    created_at timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at timestamp with time zone NOT NULL DEFAULT NOW(),
    prop_id bigint,
    schema_id bigint,
    schema_variant_id bigint,
    name text,
    internal_only bool NOT NULL DEFAULT FALSE,
    type_definition text
);
SELECT standard_model_table_constraints_v1('input_sockets');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
    VALUES ('input_sockets', 'model', 'input_socket', 'Input Socket');

CREATE OR REPLACE FUNCTION input_socket_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_prop_id bigint,
    this_schema_id bigint,
    this_schema_variant_id bigint,
    this_name text,
    this_internal_only bool,
    this_type_definition text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record           tenancy_record_v1;
    this_visibility_record        visibility_record_v1;
    this_new_row                  input_sockets%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO input_sockets (tenancy_universal,
                               tenancy_billing_account_ids,
                               tenancy_organization_ids,
                               tenancy_workspace_ids,
                               visibility_change_set_pk,
                               visibility_edit_session_pk,
                               visibility_deleted,
                               prop_id,
                               schema_id,
                               schema_variant_id,
                               name,
                               internal_only,
                               type_definition)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted,
            this_prop_id,
            this_schema_id,
            this_schema_variant_id,
            this_name,
            this_internal_only,
            this_type_definition)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
