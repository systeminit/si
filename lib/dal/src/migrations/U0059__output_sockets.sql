CREATE TABLE output_sockets
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
    attribute_context_prop_id bigint,
    attribute_context_schema_id bigint,
    attribute_context_schema_variant_id bigint,
    attribute_context_component_id bigint,
    attribute_context_system_id bigint,
    created_at timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at timestamp with time zone NOT NULL DEFAULT NOW(),
    name text,
    internal_only bool NOT NULL DEFAULT FALSE,
    type_definition text,
    source_prop_id bigint,
    transformation_func_id bigint
);
SELECT standard_model_table_constraints_v1('output_sockets');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
    VALUES ('output_sockets', 'model', 'output_socket', 'Output Socket');

CREATE OR REPLACE FUNCTION output_socket_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_context jsonb,
    this_name text,
    this_internal_only bool,
    this_source_prop_id bigint,
    this_transformation_func_id bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record           tenancy_record_v1;
    this_visibility_record        visibility_record_v1;
    this_attribute_context_record attribute_context_record_v1;
    this_new_row                  output_sockets%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := tenancy_json_to_columns_v1(this_visibility);
    this_attribute_context_record := attribute_context_json_to_columns_v1(this_attribute_context);

    INSERT INTO output_sockets (tenancy_universal,
                               tenancy_billing_account_ids,
                               tenancy_organization_ids,
                               tenancy_workspace_ids,
                               visibility_change_set_pk,
                               visibility_edit_session_pk,
                               visibility_deleted,
                               attribute_context_prop_id,
                               attribute_context_schema_id,
                               attribute_context_schema_variant_id,
                               attribute_context_component_id,
                               attribute_context_system_id,
                               name,
                               internal_only,
                               source_prop_id,
                               transformation_func_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted,
            this_attribute_context_record.attribute_context_prop_id,
            this_attribute_context_record.attribute_context_schema_id,
            this_attribute_context_record.attribute_context_schema_variant_id,
            this_attribute_context_record.attribute_context_component_id,
            this_attribute_context_record.attribute_context_system_id,
            this_name,
            this_internal_only,
            this_source_prop_id,
            this_transformation_func_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
