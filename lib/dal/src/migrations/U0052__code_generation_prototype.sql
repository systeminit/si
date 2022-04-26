CREATE TABLE code_generation_prototypes
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
    func_id                     bigint                   NOT NULL,
    args                        jsonb                    NOT NULL,
    format                      text                     NOT NULL,
    component_id                bigint                   NOT NULL,
    schema_id                   bigint                   NOT NULL,
    schema_variant_id           bigint                   NOT NULL,
    system_id                   bigint                   NOT NULL
);
SELECT standard_model_table_constraints_v1('code_generation_prototypes');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('code_generation_prototypes', 'model', 'code_generation_prototype', 'code generation Prototype');

CREATE OR REPLACE FUNCTION code_generation_prototype_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id bigint,
    this_args jsonb,
    this_format text,
    this_component_id bigint,
    this_schema_id bigint,
    this_schema_variant_id bigint,
    this_system_id bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           code_generation_prototypes%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO code_generation_prototypes (tenancy_universal,
                                       tenancy_billing_account_ids,
                                       tenancy_organization_ids,
                                       tenancy_workspace_ids,
                                       visibility_change_set_pk,
                                       visibility_edit_session_pk,
                                       visibility_deleted,
                                       func_id,
                                       args,
                                       format,
                                       component_id,
                                       schema_id,
                                       schema_variant_id,
                                       system_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted,
            this_func_id,
            this_args,
            this_format,
            this_component_id,
            this_schema_id,
            this_schema_variant_id,
            this_system_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
