CREATE TABLE attribute_prototypes
(
    pk                                     bigserial PRIMARY KEY,
    id                                     bigserial                NOT NULL,
    tenancy_universal                      bool                     NOT NULL,
    tenancy_billing_account_ids            bigint[],
    tenancy_organization_ids               bigint[],
    tenancy_workspace_ids                  bigint[],
    visibility_change_set_pk               bigint                   NOT NULL DEFAULT -1,
    visibility_edit_session_pk             bigint                   NOT NULL DEFAULT -1,
    visibility_deleted_at                  timestamp with time zone,
    attribute_context_prop_id              bigint,
    attribute_context_internal_provider_id bigint,
    attribute_context_external_provider_id bigint,
    attribute_context_schema_id            bigint,
    attribute_context_schema_variant_id    bigint,
    attribute_context_component_id         bigint,
    attribute_context_system_id            bigint,
    created_at                             timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                             timestamp with time zone NOT NULL DEFAULT NOW(),
    func_id                                bigint                   NOT NULL,
    key                                    text,
    attribute_prototype_argument_ids       bigint[]                 NOT NULL
);
SELECT standard_model_table_constraints_v1('attribute_prototypes');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('attribute_prototypes', 'model', 'attribute_prototype', 'Attribute Prototype');

CREATE OR REPLACE FUNCTION attribute_prototype_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_context jsonb,
    this_func_id bigint,
    this_key text,
    this_attribute_prototype_argument_ids bigint[],
    OUT object json) AS
$$
DECLARE
    this_tenancy_record           tenancy_record_v1;
    this_visibility_record        visibility_record_v1;
    this_attribute_context_record attribute_context_record_v1;
    this_new_row                  attribute_prototypes%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);
    this_attribute_context_record := attribute_context_json_to_columns_v1(this_attribute_context);

    INSERT INTO attribute_prototypes (tenancy_universal,
                                     tenancy_billing_account_ids,
                                     tenancy_organization_ids,
                                     tenancy_workspace_ids,
                                     visibility_change_set_pk,
                                     visibility_edit_session_pk,
                                     visibility_deleted_at,
                                     attribute_context_prop_id,
                                     attribute_context_internal_provider_id,
                                     attribute_context_external_provider_id,
                                     attribute_context_schema_id,
                                     attribute_context_schema_variant_id,
                                     attribute_context_component_id,
                                     attribute_context_system_id,
                                     func_id,
                                     key,
                                     attribute_prototype_argument_ids)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted_at,
            this_attribute_context_record.attribute_context_prop_id,
            this_attribute_context_record.attribute_context_internal_provider_id,
            this_attribute_context_record.attribute_context_external_provider_id,
            this_attribute_context_record.attribute_context_schema_id,
            this_attribute_context_record.attribute_context_schema_variant_id,
            this_attribute_context_record.attribute_context_component_id,
            this_attribute_context_record.attribute_context_system_id,
            this_func_id,
            this_key,
            this_attribute_prototype_argument_ids)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
