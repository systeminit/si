CREATE TABLE internal_providers
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint,
    visibility_edit_session_pk  bigint,
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    prop_id                     bigint                   NOT NULL,
    schema_id                   bigint                   NOT NULL,
    schema_variant_id           bigint                   NOT NULL,
    attribute_prototype_id      bigint,
    name                        text                     NOT NULL,
    internal_consumer           boolean                  NOT NULL DEFAULT TRUE,
    inbound_type_definition     text,
    outbound_type_definition    text
);

CREATE UNIQUE INDEX internal_provider_unique_index ON internal_providers (name, prop_id, schema_id, schema_variant_id);

SELECT standard_model_table_constraints_v1('internal_providers');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('internal_providers', 'model', 'internal_provider', 'Input Provider');

-- We do not want to set the attribute prototype id upon creation because we need an internal provider id for the prototype's context. --
CREATE OR REPLACE FUNCTION internal_provider_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_prop_id bigint,
    this_schema_id bigint,
    this_schema_variant_id bigint,
    this_name text,
    this_internal_consumer boolean,
    this_inbound_type_definition text,
    this_outbound_type_definition text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           internal_providers%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO internal_providers (tenancy_universal,
                                    tenancy_billing_account_ids,
                                    tenancy_organization_ids,
                                    tenancy_workspace_ids,
                                    visibility_change_set_pk,
                                    visibility_edit_session_pk,
                                    visibility_deleted_at,
                                    prop_id,
                                    schema_id,
                                    schema_variant_id,
                                    name,
                                    internal_consumer,
                                    inbound_type_definition,
                                    outbound_type_definition)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted_at,
            this_prop_id,
            this_schema_id,
            this_schema_variant_id,
            this_name,
            this_internal_consumer,
            this_inbound_type_definition,
            this_outbound_type_definition)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
